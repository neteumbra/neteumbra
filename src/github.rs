//! GitHub metadata: repo descriptions, languages, stars and the numbers in the
//! ticker. Everything is cached in `assets/profile.json` so generation always
//! works offline and a failed API call never breaks the README.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::Config;
use crate::time::{date_utc, iso_utc, now_unix};

const API: &str = "https://api.github.com";
const UA: &str = "neteumbra-profile-generator";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Cache {
    #[serde(default)]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub stats: Stats,
    #[serde(default)]
    pub repos: BTreeMap<String, RepoMeta>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stats {
    #[serde(default)]
    pub commits: Option<u32>,
    #[serde(default)]
    pub stars: Option<u32>,
    #[serde(default)]
    pub public_repos: Option<u32>,
    #[serde(default)]
    pub followers: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoMeta {
    #[serde(default)]
    pub full_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub stars: u32,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl Cache {
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(raw) => match serde_json::from_str(&raw) {
                Ok(cache) => cache,
                Err(err) => {
                    eprintln!("warn: ignoring unreadable cache ({}): {err}", path.display());
                    Cache::default()
                }
            },
            Err(_) => Cache::default(),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let raw = serde_json::to_string_pretty(self)? + "\n";
        std::fs::write(path, raw).with_context(|| format!("cannot write {}", path.display()))?;
        Ok(())
    }
}

struct Client {
    agent: ureq::Agent,
    token: Option<String>,
}

impl Client {
    fn new() -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(20))
            .user_agent(UA)
            .build();
        Self {
            agent,
            token: token_from_env(),
        }
    }

    fn get(&self, url: &str) -> Result<Value> {
        let mut req = self
            .agent
            .get(url)
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28");
        if let Some(token) = &self.token {
            req = req.set("Authorization", &format!("Bearer {token}"));
        }
        let res = req.call().map_err(|e| anyhow!("GET {url}: {e}"))?;
        res.into_json().map_err(|e| anyhow!("GET {url}: {e}"))
    }

    fn repo(&self, full_name: &str) -> Result<RepoMeta> {
        let v = self.get(&format!("{API}/repos/{full_name}"))?;
        Ok(RepoMeta {
            full_name: string_field(&v, "full_name").unwrap_or_else(|| full_name.to_string()),
            description: string_field(&v, "description"),
            language: string_field(&v, "language"),
            stars: v.get("stargazers_count").and_then(Value::as_u64).unwrap_or(0) as u32,
            homepage: string_field(&v, "homepage"),
            url: string_field(&v, "html_url")
                .unwrap_or_else(|| format!("https://github.com/{full_name}")),
            updated_at: string_field(&v, "updated_at"),
        })
    }

    fn user(&self, handle: &str) -> Result<Value> {
        self.get(&format!("{API}/users/{handle}"))
    }

    fn org(&self, org: &str) -> Result<Value> {
        self.get(&format!("{API}/orgs/{org}"))
    }

    fn repo_pages(&self, url: &str) -> Result<Vec<Value>> {
        let mut all = Vec::new();
        for page in 1..=10 {
            let sep = if url.contains('?') { '&' } else { '?' };
            let batch = self.get(&format!("{url}{sep}per_page=100&page={page}"))?;
            let items = batch
                .as_array()
                .ok_or_else(|| anyhow!("expected an array from {url}"))?;
            let len = items.len();
            all.extend(items.iter().cloned());
            if len < 100 {
                break;
            }
        }
        Ok(all)
    }

    fn commits(&self, handle: &str, window_days: u32) -> Result<u32> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| anyhow!("no token available for the GraphQL API"))?;
        let to = now_unix();
        let from = to - i64::from(window_days) * 86_400;
        let body = json!({
            "query": "query($login:String!,$from:DateTime!,$to:DateTime!){\
                user(login:$login){contributionsCollection(from:$from,to:$to){\
                totalCommitContributions}}}",
            "variables": {
                "login": handle,
                "from": format!("{}T00:00:00Z", date_utc(from)),
                "to": format!("{}T00:00:00Z", date_utc(to)),
            }
        });
        let res = self
            .agent
            .post("https://api.github.com/graphql")
            .set("Authorization", &format!("Bearer {token}"))
            .set("Accept", "application/vnd.github+json")
            .send_json(body)
            .map_err(|e| anyhow!("GraphQL: {e}"))?;
        let v: Value = res.into_json().map_err(|e| anyhow!("GraphQL: {e}"))?;
        if let Some(errors) = v.get("errors") {
            return Err(anyhow!("GraphQL errors: {errors}"));
        }
        v.pointer("/data/user/contributionsCollection/totalCommitContributions")
            .and_then(Value::as_u64)
            .map(|n| n as u32)
            .ok_or_else(|| anyhow!("GraphQL: totalCommitContributions missing"))
    }
}

/// Fetch what is available and merge it into the cache. Individual failures are
/// warnings: the previous value survives and the README keeps rendering.
pub fn fetch(cfg: &Config, cache_path: &Path) -> Result<Cache> {
    let mut cache = Cache::load(cache_path);
    let client = Client::new();
    let token_state = if client.token.is_some() {
        "authenticated"
    } else {
        "unauthenticated (60 req/h)"
    };
    println!("· fetching GitHub data ({token_state})");

    for project in &cfg.projects {
        let Some(repo) = &project.repo else { continue };
        match client.repo(repo) {
            Ok(meta) => {
                println!("  ✓ {repo}");
                cache.repos.insert(repo.clone(), meta);
            }
            Err(err) => eprintln!("  warn: {repo}: {err:#}"),
        }
    }

    let mut public_repos = 0u32;
    let mut stats_ok = false;

    match client.user(&cfg.handle) {
        Ok(user) => {
            cache.stats.followers = user
                .get("followers")
                .and_then(Value::as_u64)
                .map(|n| n as u32)
                .or(cache.stats.followers);
            public_repos += user
                .get("public_repos")
                .and_then(Value::as_u64)
                .unwrap_or(0) as u32;
            stats_ok = true;
        }
        Err(err) => eprintln!("  warn: user {}: {err:#}", cfg.handle),
    }

    let mut star_total = 0u32;
    match client.repo_pages(&format!(
        "{API}/users/{}/repos?type=owner",
        cfg.handle
    )) {
        Ok(repos) => {
            star_total += sum_stars(&repos);
            stats_ok = true;
        }
        Err(err) => eprintln!("  warn: user repos: {err:#}"),
    }

    for org in &cfg.orgs {
        match client.org(org) {
            Ok(v) => {
                public_repos += v
                    .get("public_repos")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as u32;
                stats_ok = true;
            }
            Err(err) => eprintln!("  warn: org {org}: {err:#}"),
        }
        match client.repo_pages(&format!("{API}/orgs/{org}/repos?type=public")) {
            Ok(repos) => {
                star_total += sum_stars(&repos);
                stats_ok = true;
            }
            Err(err) => eprintln!("  warn: org repos {org}: {err:#}"),
        }
    }

    if stats_ok {
        cache.stats.stars = Some(star_total);
        cache.stats.public_repos = Some(public_repos);
    }

    match client.commits(&cfg.handle, cfg.commit_window_days) {
        Ok(commits) => cache.stats.commits = Some(commits),
        Err(err) => eprintln!("  warn: commits: {err:#}"),
    }

    cache.generated_at = Some(iso_utc(now_unix()));
    cache.save(cache_path)?;
    println!("· cache written to {}", cache_path.display());
    Ok(cache)
}

fn sum_stars(repos: &[Value]) -> u32 {
    repos
        .iter()
        .map(|r| r.get("stargazers_count").and_then(Value::as_u64).unwrap_or(0) as u32)
        .sum()
}

fn string_field(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn token_from_env() -> Option<String> {
    ["STATS_TOKEN", "GITHUB_TOKEN", "GH_TOKEN"]
        .iter()
        .find_map(|key| std::env::var(key).ok())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_roundtrips_and_tolerates_missing_fields() {
        let raw = r#"{"stats":{"stars":3},"repos":{"a/b":{"stars":2,"full_name":"a/b"}}}"#;
        let cache: Cache = serde_json::from_str(raw).unwrap();
        assert_eq!(cache.stats.stars, Some(3));
        assert_eq!(cache.repos["a/b"].stars, 2);
        assert_eq!(cache.stats.commits, None);
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let raw = r#"{"generated_at":null,"stats":{},"repos":{},"future":"ok"}"#;
        assert!(serde_json::from_str::<Cache>(raw).is_ok());
    }

    #[test]
    fn sums_stars() {
        let repos = vec![json!({"stargazers_count": 2}), json!({"stargazers_count": 5})];
        assert_eq!(sum_stars(&repos), 7);
    }

    #[test]
    fn blank_strings_are_none() {
        let v = json!({"description":"  ", "homepage": null, "language": "Rust"});
        assert_eq!(string_field(&v, "description"), None);
        assert_eq!(string_field(&v, "homepage"), None);
        assert_eq!(string_field(&v, "language").as_deref(), Some("Rust"));
    }
}
