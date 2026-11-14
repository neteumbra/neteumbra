//! Hero: the first screen. A terminal boot sequence on the right, a glitched
//! display title on the left, HUD chrome around the whole thing.

use crate::config::Config;
use crate::github::Cache;
use crate::svg::*;

const W: f64 = 900.0;
const H: f64 = 430.0;

pub fn render(cfg: &Config, cache: &Cache) -> String {
    let sync = cache
        .generated_at
        .as_deref()
        .and_then(|s| s.get(0..7))
        .unwrap_or("LIVE");

    let mut body = String::new();
    body.push_str(&format!(
        r#"<rect width="{W}" height="{H}" fill="{BG}"/><rect width="{W}" height="{H}" fill="url(#glow)"/><rect width="{W}" height="{H}" fill="url(#grid)"/><rect width="{W}" height="{H}" fill="url(#scanlines)"/>"#
    ));
    body.push_str(&format!(
        r#"<rect x="0" y="0" width="{W}" height="150" fill="url(#scanband)" class="scan"/>"#
    ));
    body.push_str(&corner_brackets(W, H, 14.0, 20.0));

    // Top meta line.
    body.push_str(&format!(
        r#"<text x="48" y="42" font-size="11" fill="{ACCENT}" letter-spacing="3">{}</text>"#,
        esc(&cfg.kicker)
    ));
    body.push_str(&format!(
        r#"<text x="852" y="42" font-size="10" fill="{MUTED_2}" letter-spacing="1.6" text-anchor="end">[ {} ]</text>"#,
        esc(&cfg.location)
    ));

    // Display title with glitch ghosts.
    let role_a = cfg.role.first().cloned().unwrap_or_default().to_uppercase();
    let role_b = cfg.role.get(1).cloned().unwrap_or_default().to_uppercase();
    body.push_str(&format!(
        r#"<text x="48" y="154" font-size="62" font-weight="800" fill="{INK}" letter-spacing="2" textLength="430" lengthAdjust="spacing">{}</text>"#,
        esc(&role_a)
    ));
    for (class, fill, opacity) in [("gA", ACCENT, ".55"), ("gB", INK, ".35")] {
        body.push_str(&format!(
            r#"<text class="{class}" x="48" y="216" font-size="62" font-weight="800" fill="{fill}" fill-opacity="{opacity}" letter-spacing="2" textLength="430" lengthAdjust="spacing">{}</text>"#,
            esc(&role_b)
        ));
    }
    body.push_str(&format!(
        r#"<text x="48" y="216" font-size="62" font-weight="800" fill="none" stroke="{ACCENT}" stroke-width="1.7" letter-spacing="2" textLength="430" lengthAdjust="spacing">{}</text>"#,
        esc(&role_b)
    ));

    // Tagline + rule.
    body.push_str(&format!(
        r#"<text x="48" y="252" font-size="15" fill="{MUTED}" letter-spacing=".4">{}</text>"#,
        esc(&cfg.tagline)
    ));
    body.push_str(&format!(
        r#"<rect x="48" y="272" width="472" height="1" fill="{LINE}"/><rect x="48" y="272" width="64" height="1" fill="{ACCENT}"/>"#
    ));
    body.push_str(&format!(
        r#"<text x="48" y="294" font-size="10.5" fill="{ACCENT}" letter-spacing="2.4">SECURITY BY DEFAULT</text><text x="242" y="294" font-size="10.5" fill="{MUTED_2}" letter-spacing="1.6">· PORTFOLIO 2026</text>"#
    ));

    // Status pill.
    body.push_str(&format!(
        r#"<rect x="48" y="314" width="258" height="30" rx="15" fill="{ACCENT}" fill-opacity=".08" stroke="{ACCENT}" stroke-opacity=".45"/><circle cx="68" cy="329" r="4" fill="{ACCENT}" class="pulse"/><text x="82" y="333" font-size="10.5" fill="{ACCENT}" letter-spacing="1.5">{}</text>"#,
        esc(&cfg.status.to_uppercase())
    ));

    // Stack chips.
    let mut x = 48.0;
    for item in cfg.stack.iter().take(4) {
        let w = chip_width(item, 20.0);
        body.push_str(&chip(x, 366.0, w, 24.0, item, false));
        x += w + 8.0;
    }

    body.push_str(&terminal(cfg, sync));

    // Bottom axis: quietly link the visual language to the site.
    body.push_str(&format!(
        r#"<text x="852" y="410" font-size="9" fill="{MUTED_2}" letter-spacing="1.4" text-anchor="end">GITHUB.COM/{}</text>"#,
        esc(&cfg.handle.to_uppercase())
    ));

    let alt = alt(cfg);
    doc(W, H, &alt, &body, &css(), &defs(), Some(16.0))
}

pub fn alt(cfg: &Config) -> String {
    let role_a = cfg.role.first().cloned().unwrap_or_default().to_uppercase();
    let role_b = cfg.role.get(1).cloned().unwrap_or_default().to_uppercase();
    format!(
        "{} — {} {}. {}. {}.",
        cfg.name,
        title_case(&role_a),
        title_case(&role_b),
        cfg.tagline,
        sentence_case(&cfg.status)
    )
}

fn terminal(cfg: &Config, sync: &str) -> String {
    let px = 560.0;
    let py = 52.0;
    let pw = 300.0;
    let ph = 330.0;
    let lx = px + 16.0;
    let lh = 22.0;
    let mut out = String::new();

    out.push_str(&format!(
        r#"<g class="rise" style="animation-delay:.15s"><rect x="{px}" y="{py}" width="{pw}" height="{ph}" rx="12" fill="{SURFACE}" stroke="{LINE_STRONG}"/><path d="M{px} {ty}H{px2}V{py2}H{px}Z" fill="{SURFACE_2}"/>"#,
        ty = py + 32.0,
        px2 = px + pw,
        py2 = py + 32.0,
    ));
    out.push_str(&format!(
        r#"<rect x="{px}" y="{ty}" width="{pw}" height="1" fill="{LINE}"/>"#,
        ty = py + 32.0
    ));
    for i in 0..3 {
        out.push_str(&format!(
            r#"<circle cx="{cx}" cy="{cy}" r="3.4" fill="{MUTED_2}"/>"#,
            cx = px + 16.0 + i as f64 * 12.0,
            cy = py + 16.0
        ));
    }
    out.push_str(&format!(
        r#"<text x="{tx}" y="{ty}" font-size="9" fill="{MUTED_2}" letter-spacing=".8" text-anchor="middle">{}@{}: ~</text>"#,
        esc(&cfg.handle),
        esc(&cfg.brand.to_lowercase()),
        tx = px + pw / 2.0 + 14.0,
        ty = py + 20.0
    ));

    let mut y = py + 56.0;
    out.push_str(&command("whoami", lx, y, "c1"));
    y += lh;
    out.push_str(&format!(
        r#"<text class="rise" style="animation-delay:.55s" x="{lx}" y="{y:.1}" font-size="11" fill="{MUTED}" letter-spacing=".4">{}</text>"#,
        esc(&cfg.name.to_lowercase())
    ));
    y += lh;

    out.push_str(&command("cat stack.txt", lx, y, "c3"));
    y += lh;
    for (n, pair) in cfg.stack.chunks(2).take(2).enumerate() {
        let left = pair.first().cloned().unwrap_or_default();
        let right = pair.get(1).cloned().unwrap_or_default();
        out.push_str(&format!(
            r#"<text class="rise" style="animation-delay:{delay:.2}s" x="{lx}" y="{y:.1}" font-size="10.5" fill="{MUTED_2}" letter-spacing=".6">{left}</text><text class="rise" style="animation-delay:{delay:.2}s" x="{rx}" y="{y:.1}" font-size="10.5" fill="{MUTED_2}" letter-spacing=".6">{right}</text>"#,
            delay = 1.25 + n as f64 * 0.1,
            left = esc(&truncate(&left, 13)),
            right = esc(&truncate(&right, 13)),
            rx = lx + 122.0
        ));
        y += lh;
    }

    out.push_str(&command("systemctl status profile", lx, y, "c7"));
    y += lh;
    out.push_str(&format!(
        r#"<text class="rise" style="animation-delay:2.35s" x="{lx}" y="{y:.1}" font-size="10.5" fill="{ACCENT}">●</text><text class="rise" style="animation-delay:2.35s" x="{tx}" y="{y:.1}" font-size="10.5" fill="{INK}" letter-spacing=".4">active (running)</text>"#,
        tx = lx + 14.0
    ));
    y += lh;

    out.push_str(&command("echo $NEXT", lx, y, "c9"));
    y += lh;
    out.push_str(&format!(
        r#"<text class="rise" style="animation-delay:2.95s" x="{lx}" y="{y:.1}" font-size="10.5" fill="{ACCENT}" letter-spacing=".4">open to freelance work</text>"#
    ));

    // Blinking cursor after the last output.
    out.push_str(&format!(
        r#"<rect class="blink" x="{lx}" y="{cy:.1}" width="7" height="13" fill="{ACCENT}"/>"#,
        cy = y + 8.0
    ));

    // Terminal footer.
    out.push_str(&format!(
        r#"<rect x="{x}" y="{fy:.1}" width="{w}" height="1" fill="{LINE}"/><text x="{lx}" y="{ty:.1}" font-size="8.5" fill="{MUTED_2}" letter-spacing="1.1">UTF-8 · BASH · SYNC {sync}</text><text x="{tx:.1}" y="{ty:.1}" font-size="8.5" fill="{ACCENT}" letter-spacing="1.1" text-anchor="end">[ 200 OK ]</text></g>"#,
        x = px + 16.0,
        w = pw - 32.0,
        fy = py + ph - 46.0,
        ty = py + ph - 26.0,
        tx = px + pw - 16.0
    ));
    out
}

fn command(cmd: &str, x: f64, y: f64, id: &str) -> String {
    format!(
        r#"<g clip-path="url(#{id})"><text x="{x}" y="{y:.1}" font-size="11" fill="{ACCENT}" letter-spacing=".4">$</text><text x="{cx:.1}" y="{y:.1}" font-size="11" fill="{INK}" letter-spacing=".4">{cmd}</text></g>"#,
        cx = x + 13.0,
        cmd = esc(cmd)
    )
}

fn defs() -> String {
    let mut defs = dark_defs();
    for (id, width) in [("c1", 84.0), ("c3", 112.0), ("c7", 196.0), ("c9", 116.0)] {
        defs.push_str(&format!(
            r#"<clipPath id="{id}"><rect class="t{id}" x="576" y="95" width="240" height="18" style="--w1:{width}px"/></clipPath>"#
        ));
    }
    defs
}

fn css() -> String {
    r#"
.tc1{animation:type1 .45s steps(9) both .25s}
.tc3{animation:type1 .45s steps(12) both .85s}
.tc7{animation:type1 .55s steps(20) both 1.75s}
.tc9{animation:type1 .45s steps(11) both 2.55s}
"#
    .to_string()
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
        assert!(svg.contains("viewBox=\"0 0 900 430\""));
        assert!(svg.contains("aria-label="));
        assert!(svg.contains("NETEUMBRA"));
        assert!(!svg.contains("<script"));
    }

    #[test]
    fn every_typed_line_has_a_clip() {
        let svg = render(&config(), &Cache::default());
        for id in ["c1", "c3", "c7", "c9"] {
            assert!(svg.contains(&format!("clip-path=\"url(#{id})\"")), "{id}");
            assert!(svg.contains(&format!("id=\"{id}\"")), "{id}");
        }
    }
}
