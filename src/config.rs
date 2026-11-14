//! `profile.config.json` — the single source of truth.
//!
//! The config is intentionally permissive: adding a project is just adding an
//! `owner/repo` string. Anything the GitHub API does not know about (a WIP with
//! no public repo, a custom tagline) can be typed in by hand and wins over the
//! fetched metadata.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::github::{Cache, RepoMeta};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub handle: String,
    pub brand: String,
    pub name: String,
    pub role: Vec<String>,
    pub tagline: String,
    #[serde(default = "default_kicker")]
    pub kicker: String,
    #[serde(default = "default_location")]
    pub location: String,
    pub status: String,
    pub about: String,
    #[serde(default)]
    pub stack: Vec<String>,
    #[serde(default)]
    pub orgs: Vec<String>,
    #[serde(default = "default_window")]
    pub commit_window_days: u32,
    #[serde(default)]
    pub projects: Vec<ProjectCfg>,
    #[serde(default)]
    pub signals: Vec<SignalCfg>,
    #[serde(default)]
    pub contacts: Vec<ContactCfg>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectCfg {
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub live: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub wip: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignalCfg {
    pub label: String,
    pub title: String,
    pub value: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContactCfg {
    pub label: String,
    pub url: String,
    #[serde(default)]
    pub filled: bool,
}

fn default_kicker() -> String {
    "PROFILE // 001".into()
}

fn default_location() -> String {
    "REMOTE".into()
}

fn default_window() -> u32 {
    365
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read {}", path.display()))?;
        let cfg: Config = serde_json::from_str(&raw)
            .with_context(|| format!("invalid JSON in {}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            self.role.len() == 2,
            "`role` must contain exactly two lines (e.g. [\"FULLSTACK\", \"DEVELOPER\"])"
        );
        anyhow::ensure!(!self.name.trim().is_empty(), "`name` cannot be empty");
        anyhow::ensure!(!self.about.trim().is_empty(), "`about` cannot be empty");
        anyhow::ensure!(
            self.projects
                .iter()
                .any(|p| p.repo.is_some() || p.label.is_some()),
            "`projects` needs at least one entry with a repo or label"
        );
        Ok(())
    }

    /// Turn the config plus the API cache into the rows that actually render.
    /// Config values always win over fetched metadata.
    pub fn projects(&self, cache: &Cache) -> Vec<ResolvedProject> {
        self.projects
            .iter()
            .filter_map(|p| resolve_project(p, cache))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProject {
    pub label: String,
    pub description: String,
    pub tag: String,
    pub repo_url: Option<String>,
    pub live_url: Option<String>,
    pub stars: Option<u32>,
    pub wip: bool,
}

impl ResolvedProject {
    /// URL the whole row points at: the source repo, or the configured URL for
    /// entries that do not live on GitHub.
    pub fn primary_url(&self) -> Option<&str> {
        self.repo_url.as_deref().or(self.live_url.as_deref())
    }
}

fn resolve_project(cfg: &ProjectCfg, cache: &Cache) -> Option<ResolvedProject> {
    let meta: Option<&RepoMeta> = cfg.repo.as_ref().and_then(|r| cache.repos.get(r));

    let label = cfg
        .label
        .clone()
        .or_else(|| {
            cfg.repo
                .as_ref()
                .map(|r| r.rsplit('/').next().unwrap_or(r).to_string())
        })
        .or_else(|| meta.map(|m| m.full_name.clone()))
        .filter(|l| !l.trim().is_empty())?;

    let mut description = cfg
        .description
        .clone()
        .or_else(|| meta.and_then(|m| m.description.clone()))
        .unwrap_or_default();
    description = description.trim().to_string();
    if description.is_empty() {
        description = "No description yet — open the repo to see what is inside.".to_string();
    }

    let tag = cfg
        .tag
        .clone()
        .or_else(|| meta.and_then(|m| m.language.clone()))
        .or_else(|| {
            if cfg.tags.is_empty() {
                None
            } else {
                Some(cfg.tags.join(" · "))
            }
        })
        .unwrap_or_else(|| "CODE".to_string());

    let repo_url = cfg
        .repo
        .as_ref()
        .map(|r| format!("https://github.com/{r}"));

    let live_url = cfg
        .live
        .clone()
        .or_else(|| meta.and_then(|m| m.homepage.clone()))
        .filter(|u| !u.trim().is_empty());

    let wip = cfg.wip
        || description_is_wip(&description)
        || meta.is_some_and(|m| description_is_wip(m.description.as_deref().unwrap_or("")));

    Some(ResolvedProject {
        label,
        description,
        tag,
        repo_url,
        live_url,
        stars: meta.map(|m| m.stars),
        wip,
    })
}

fn description_is_wip(description: &str) -> bool {
    let d = description.to_lowercase();
    ["en desarrollo", "in development", "work in progress", "wip"]
        .iter()
        .any(|needle| d.contains(needle))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSignal {
    pub label: String,
    pub title: String,
    pub value: String,
    pub description: String,
    pub url: Option<String>,
}

impl Config {
    pub fn signals(&self) -> Vec<ResolvedSignal> {
        self.signals
            .iter()
            .map(|s| ResolvedSignal {
                label: s.label.clone(),
                title: s.title.clone(),
                value: s.value.clone(),
                description: s.description.clone(),
                url: s.url.clone().filter(|u| !u.trim().is_empty()),
            })
            .collect()
    }
}

/// Stable filename fragment: "NeTenebrae-Sec/neCyberWiki" → "necyberwiki".
pub fn slug(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            dash = false;
        } else if !dash {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Config {
        serde_json::from_str(
            r#"{
              "handle":"neteumbra","brand":"NETEUMBRA","name":"Mauricio",
              "role":["FULLSTACK","DEVELOPER"],"tagline":"Java","status":"OPEN",
              "about":"Hello","projects":[{"repo":"a/b"}],
              "contacts":[{"label":"GITHUB","url":"https://github.com/x"}]
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn parses_minimal_config() {
        let cfg = base();
        assert_eq!(cfg.role, ["FULLSTACK", "DEVELOPER"]);
        assert_eq!(cfg.commit_window_days, 365);
        assert_eq!(cfg.kicker, "PROFILE // 001");
    }

    #[test]
    fn config_overrides_fetched_metadata() {
        let cfg = base();
        let mut cache = Cache::default();
        cache.repos.insert(
            "a/b".into(),
            RepoMeta {
                full_name: "a/b".into(),
                description: Some("API description".into()),
                language: Some("Go".into()),
                stars: 7,
                homepage: Some("https://b.dev".into()),
                url: "https://github.com/a/b".into(),
                updated_at: None,
            },
        );
        let p = &cfg.projects(&cache)[0];
        assert_eq!(p.label, "b");
        assert_eq!(p.description, "API description");
        assert_eq!(p.tag, "Go");
        assert_eq!(p.stars, Some(7));
        assert_eq!(p.live_url.as_deref(), Some("https://b.dev"));
    }

    #[test]
    fn static_project_without_repo_renders() {
        let cfg: Config = serde_json::from_str(
            r#"{
              "handle":"x","brand":"X","name":"X","role":["A","B"],"tagline":"t",
              "status":"s","about":"a",
              "projects":[{"label":"SMT","description":"Compendium","wip":true}],
              "contacts":[]
            }"#,
        )
        .unwrap();
        let p = &cfg.projects(&Cache::default())[0];
        assert_eq!(p.label, "SMT");
        assert_eq!(p.stars, None);
        assert!(p.wip);
        assert_eq!(p.primary_url(), None);
    }

    #[test]
    fn missing_description_gets_a_readable_fallback() {
        let cfg = base();
        let p = &cfg.projects(&Cache::default())[0];
        assert!(p.description.starts_with("No description yet"));
        assert_eq!(p.tag, "CODE");
    }

    #[test]
    fn detects_wip_from_description() {
        assert!(description_is_wip("Mi entorno | EN DESARROLLO"));
        assert!(description_is_wip("work in progress"));
        assert!(!description_is_wip("A finished product"));
    }

    #[test]
    fn slugs_are_stable() {
        assert_eq!(slug("NeTenebrae-Sec/neCyberWiki"), "netenebrae-sec-necyberwiki");
        assert_eq!(slug("GITHUB"), "github");
        assert_eq!(slug("  a  b  "), "a-b");
    }

    #[test]
    fn rejects_wrong_role_shape() {
        let mut cfg = base();
        cfg.role.pop();
        assert!(cfg.validate().is_err());
    }
}
