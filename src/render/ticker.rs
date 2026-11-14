//! Bottom ticker: the portfolio's marquee, fed with live numbers when the cache
//! has them.

use crate::config::Config;
use crate::github::Cache;
use crate::svg::*;

const W: f64 = 900.0;
const H: f64 = 44.0;

pub fn render(cfg: &Config, cache: &Cache) -> (String, String) {
    let mut parts: Vec<String> = vec![
        cfg.role.join(" "),
        "SECURITY BY DEFAULT".into(),
    ];
    parts.extend(cfg.stack.iter().cloned());
    if let Some(commits) = cache.stats.commits {
        let months = (cfg.commit_window_days / 30).max(1);
        parts.push(format!("{} COMMITS · {months}M", fmt_num(commits)));
    }
    if let Some(stars) = cache.stats.stars {
        parts.push(format!("{} STARS", fmt_num(stars)));
    }
    if let Some(repos) = cache.stats.public_repos {
        parts.push(format!("{} PUBLIC REPOS", fmt_num(repos)));
    }
    if let Some(followers) = cache.stats.followers {
        parts.push(format!("{} FOLLOWERS", fmt_num(followers)));
    }
    if let Some(stamp) = cache.generated_at.as_deref().and_then(|s| s.get(0..10)) {
        parts.push(format!("SYNC {stamp}"));
    }
    parts.push("THANKS FOR STOPPING BY".into());

    let unit = format!("{}   ★   ", parts.join("   ·   "));
    let size = 11.0;
    let ls = 1.2;
    let unit_w = text_width(&unit, size, ls);
    let duration = (unit_w / 42.0).max(18.0);

    let mut body = String::new();
    body.push_str(&format!(
        r#"<rect width="{W}" height="{H}" fill="{SURFACE}"/><rect x="0" y="0" width="{W}" height="1" fill="{ACCENT}" fill-opacity=".55"/><rect x="0" y="{y}" width="{W}" height="1" fill="{ACCENT}" fill-opacity=".55"/><clipPath id="mq"><rect x="16" y="0" width="{cw}" height="{H}"/></clipPath>"#,
        y = H - 1.0,
        cw = W - 32.0
    ));
    body.push_str(&format!(
        r#"<g clip-path="url(#mq)"><g class="mq" style="--mq-shift:-{unit_w:.1}px;--mq-dur:{duration:.1}s">"#
    ));
    for k in 0..3 {
        body.push_str(&format!(
            r#"<text x="{x:.1}" y="27" font-size="{size}" fill="{ACCENT}" fill-opacity=".85" letter-spacing="{ls}" textLength="{unit_w:.1}" lengthAdjust="spacing">{text}</text>"#,
            x = 18.0 + k as f64 * unit_w,
            text = esc(&unit)
        ));
    }
    body.push_str("</g></g>");

    let svg = doc(W, H, &unit, &body, "", "", None);
    (svg, unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        serde_json::from_str(include_str!("../../profile.config.json")).unwrap()
    }

    #[test]
    fn ticker_uses_cached_stats_only() {
        let (svg, alt) = render(&config(), &Cache::default());
        assert!(!svg.contains("COMMITS"));
        assert!(svg.contains("SECURITY BY DEFAULT"));
        assert!(svg.contains("THANKS FOR STOPPING BY"));
        assert!(alt.contains("★"));
    }

    #[test]
    fn ticker_includes_numbers_when_present() {
        let mut cache = Cache::default();
        cache.stats.commits = Some(1_234);
        cache.stats.stars = Some(29);
        cache.stats.public_repos = Some(34);
        cache.stats.followers = Some(14);
        cache.generated_at = Some("2026-11-13T00:00:00Z".into());
        let (svg, _) = render(&config(), &cache);
        assert!(svg.contains("1,234 COMMITS"));
        assert!(svg.contains("29 STARS"));
        assert!(svg.contains("34 PUBLIC REPOS"));
        assert!(svg.contains("14 FOLLOWERS"));
        assert!(svg.contains("SYNC 2026-11-13"));
    }
}
