//! Hero: the first screen. Deliberately quiet — a large name, the role in
//! outline, one tagline and one status line. No terminal, no chips, no ghost
//! layers: everything that is not identity lives in the sections below.

use crate::config::Config;
use crate::github::Cache;
use crate::svg::*;

const W: f64 = 900.0;
const H: f64 = 300.0;

pub fn render(cfg: &Config, _cache: &Cache) -> String {
    let name = cfg.name.to_uppercase();
    let role = cfg.role.join(" ").to_uppercase();
    let status = cfg.status.to_uppercase();

    let mut body = String::new();
    body.push_str(&format!(
        r#"<rect width="{W}" height="{H}" fill="{BG}"/><rect width="{W}" height="{H}" fill="url(#glow)"/>"#
    ));
    body.push_str(&corner_brackets(W, H, 16.0, 22.0));

    // Top rail: one label, one side. The right side stays empty on purpose.
    body.push_str(&format!(
        r#"<text x="48" y="38" font-size="10" fill="{ACCENT}" letter-spacing="2.8">{}</text>"#,
        esc(&cfg.kicker.to_uppercase())
    ));

    // Name.
    body.push_str(&format!(
        r#"<text class="rise" x="48" y="136" font-size="76" font-weight="800" fill="{INK}" letter-spacing="3">{}</text>"#,
        esc(&name)
    ));

    // Role, outlined.
    body.push_str(&format!(
        r#"<text class="rise" style="animation-delay:.14s" x="48" y="192" font-size="48" font-weight="800" fill="none" stroke="{ACCENT}" stroke-width="1.5" letter-spacing="3">{}</text>"#,
        esc(&role)
    ));

    // Short accent rule.
    body.push_str(&format!(
        r#"<rect class="fade" style="animation-delay:.3s" x="48" y="210" width="88" height="2" fill="{ACCENT}"/>"#
    ));

    // Tagline.
    body.push_str(&format!(
        r#"<text class="rise" style="animation-delay:.36s" x="48" y="240" font-size="14.5" fill="{MUTED}" letter-spacing=".3">{}</text>"#,
        esc(&cfg.tagline)
    ));

    // Status + handle.
    body.push_str(&format!(
        r#"<circle class="pulse" cx="52" cy="258" r="4" fill="{ACCENT}"/><text x="66" y="262" font-size="10.5" fill="{ACCENT}" letter-spacing="1.8">{}</text><text x="852" y="262" font-size="9.5" fill="{MUTED_2}" letter-spacing="1.6" text-anchor="end">GITHUB.COM/{}</text>"#,
        esc(&status),
        esc(&cfg.handle.to_uppercase())
    ));

    let alt = alt(cfg);
    doc(W, H, &alt, &body, "", &defs(), Some(16.0))
}

pub fn alt(cfg: &Config) -> String {
    format!(
        "{} — {} {}. {}. {}.",
        cfg.name,
        title_case(cfg.role.first().map(String::as_str).unwrap_or("")),
        title_case(cfg.role.get(1).map(String::as_str).unwrap_or("")),
        cfg.tagline,
        sentence_case(&cfg.status)
    )
}

fn defs() -> String {
    format!(
        r#"<radialGradient id="glow" cx="78%" cy="-12%" r="88%"><stop offset="0" stop-color="{ACCENT}" stop-opacity=".15"/><stop offset=".55" stop-color="{ACCENT}" stop-opacity=".04"/><stop offset="1" stop-color="{ACCENT}" stop-opacity="0"/></radialGradient>"#
    )
}

fn title_case(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut chars = lower.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn sentence_case(s: &str) -> String {
    title_case(&s.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn config() -> Config {
        serde_json::from_str(include_str!("../../profile.config.json")).unwrap()
    }

    #[test]
    fn hero_stays_inside_its_frame() {
        let svg = render(&config(), &Cache::default());
        assert!(svg.contains("viewBox=\"0 0 900 300\""));
        assert!(svg.contains("aria-label="));
        assert!(svg.contains("MAURICIO"));
        assert!(!svg.contains("<script"));
    }

    #[test]
    fn hero_is_just_identity() {
        let svg = render(&config(), &Cache::default());
        for needle in [
            "whoami",
            "cat stack.txt",
            "systemctl",
            "UTC-3",
            "PORTFOLIO 2026",
            "SECURITY BY DEFAULT",
        ] {
            assert!(!svg.contains(needle), "{needle} should not be in the hero");
        }
        assert!(svg.contains("FULLSTACK DEVELOPER"));
        assert!(svg.contains("GITHUB.COM/NETEUMBRA"));
        assert!(svg.contains("AVAILABLE FOR NEW PROJECTS"));
        assert!(svg.contains("prefers-reduced-motion"));
    }
}
