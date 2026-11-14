//! About: a terminal window that prints the short bio, line by line.

use crate::config::Config;
use crate::svg::*;

const W: f64 = 900.0;
const PAD_X: f64 = 40.0;
const SIZE: f64 = 14.0;

pub fn render(cfg: &Config) -> String {
    let lines = wrap(&cfg.about, 92, 5);
    let h = 82.0 + lines.len() as f64 * 24.0;
    let mut body = String::new();

    body.push_str(&format!(
        r#"<rect width="{W}" height="{h}" rx="12" fill="{SURFACE}" stroke="{LINE_STRONG}"/><path d="M0 34H{W}V0H0Z" fill="{SURFACE_2}"/><rect x="0" y="34" width="{W}" height="1" fill="{LINE}"/>"#
    ));
    for i in 0..3 {
        body.push_str(&format!(
            r#"<circle cx="{cx}" cy="17" r="3.4" fill="{MUTED_2}"/>"#,
            cx = 18.0 + i as f64 * 12.0
        ));
    }
    body.push_str(&format!(
        r#"<text x="70" y="21" font-size="9.5" fill="{MUTED_2}" letter-spacing="1">~/about.md</text><text x="860" y="21" font-size="9" fill="{ACCENT}" letter-spacing="1.4" text-anchor="end">[ READ ]</text>"#
    ));

    body.push_str(&format!(
        r#"<text x="{PAD_X}" y="62" font-size="12" fill="{ACCENT}" letter-spacing=".5">$</text><text x="{tx}" y="62" font-size="12" fill="{INK}" letter-spacing=".5">cat about.md</text>"#,
        tx = PAD_X + 14.0
    ));

    let first_y = 90.0;
    body.push_str(&format!(
        r#"<rect x="22" y="{y}" width="2" height="{h2}" fill="{ACCENT}" fill-opacity=".55"/>"#,
        y = first_y - 17.0,
        h2 = lines.len() as f64 * 24.0 - 2.0
    ));

    for (i, line) in lines.iter().enumerate() {
        let y = first_y + i as f64 * 24.0;
        body.push_str(&format!(
            r#"<text class="rise" style="animation-delay:{delay:.2}s" x="{PAD_X}" y="{y:.1}" font-size="{SIZE}" fill="{INK}" letter-spacing=".1">{}</text>"#,
            esc(line),
            delay = 0.15 + i as f64 * 0.1
        ));
    }

    // Cursor right after the final line.
    let last = lines.last().cloned().unwrap_or_default();
    let cursor_x = PAD_X + text_width(&last, SIZE, 0.1) + 7.0;
    let last_y = first_y + (lines.len().saturating_sub(1)) as f64 * 24.0;
    body.push_str(&format!(
        r#"<rect class="blink" x="{cursor_x:.1}" y="{y:.1}" width="7" height="14" fill="{ACCENT}"/>"#,
        y = last_y - 11.0
    ));

    doc(W, h, &cfg.about, &body, "", "", Some(12.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(about: &str) -> Config {
        let raw = format!(
            r#"{{"handle":"h","brand":"B","name":"N","role":["A","B"],"tagline":"t",
                "status":"s","about":{about},"projects":[{{"label":"X"}}],"contacts":[]}}"#
        );
        serde_json::from_str(&raw).unwrap()
    }

    #[test]
    fn height_grows_with_the_text() {
        let short = render(&config("\"one line\""));
        let long = render(&config(
            "\"This is a much longer biography that should wrap across several lines and make the panel taller than the short one.\"",
        ));
        assert!(short.contains("height=\"106\""));
        assert!(long.contains("height=\"130\""), "{long}");
    }

    #[test]
    fn very_long_bios_are_capped() {
        let text = "word ".repeat(400);
        let svg = render(&config(&format!("\"{text}\"")));
        assert!(svg.contains("height=\"202\""), "caps at five lines");
        assert!(svg.contains('…'));
    }
}
