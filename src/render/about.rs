//! About: an editorial spread. No window chrome, no prompt, no cursor: a lead
//! line, a hairline and two measured columns of body text.

use crate::config::Config;
use crate::svg::*;

const W: f64 = 900.0;
const PAD_X: f64 = 48.0;
const COL_W: f64 = 350.0;
const COL_GAP: f64 = 104.0;
const LEAD_SIZE: f64 = 23.0;
const LEAD_LINE_H: f64 = 31.0;
const BODY_SIZE: f64 = 14.0;
const BODY_LINE_H: f64 = 21.0;
const BODY_MAX_LINES: usize = 4;

/// Neutral alt: the channel name is intentionally absent so it is not indexed
/// as page text. It still lives inside the rendered SVG, which is an image.
pub fn alt(cfg: &Config) -> String {
    format!(
        "01 / About {} — {}. Web, APIs, Linux and security; also a Spanish-language hacking and Linux channel.",
        cfg.name,
        cfg.role.join(" ").to_lowercase()
    )
}

pub fn render(cfg: &Config) -> String {
    let (lead, rest) = split_lead(&cfg.about);
    let lead_chars = fit_chars(W - PAD_X * 2.0, LEAD_SIZE, 0.2);
    let lead_lines = wrap(lead, lead_chars, 4);

    let (left_text, right_text) = split_body(rest);
    let body_chars = fit_chars(COL_W, BODY_SIZE, 0.1);
    let left = wrap(&left_text, body_chars, BODY_MAX_LINES);
    let right = wrap(&right_text, body_chars, BODY_MAX_LINES);
    let rows = if left_text.is_empty() {
        0
    } else {
        left.len().max(right.len())
    };

    let lead_bottom = 100.0 + (lead_lines.len() as f64 - 1.0) * LEAD_LINE_H;
    let rule_y = lead_bottom + 23.0;
    let body_top = rule_y + 26.0;
    let h = if rows == 0 {
        rule_y + 28.0
    } else {
        body_top + (rows as f64 - 1.0) * BODY_LINE_H + 26.0
    };

    let mut body = String::new();
    body.push_str(&format!(
        r#"<rect width="{W}" height="{h}" fill="{BG}"/><text x="{PAD_X}" y="34" font-size="10" fill="{ACCENT}" letter-spacing="2.4">01 / ABOUT</text><text x="852" y="34" font-size="9" fill="{MUTED_2}" letter-spacing="1.6" text-anchor="end">[ {} ]</text><rect x="{PAD_X}" y="48" width="{}" height="1" fill="{LINE}"/><rect x="{PAD_X}" y="48" width="54" height="1" fill="{ACCENT}"/><rect x="{PAD_X}" y="66" width="44" height="2" fill="{ACCENT}"/>"#,
        esc(&cfg.role.join(" ")),
        W - PAD_X * 2.0
    ));

    for (i, line) in lead_lines.iter().enumerate() {
        let y = 100.0 + i as f64 * LEAD_LINE_H;
        body.push_str(&format!(
            r#"<text class="rise" style="animation-delay:{:.2}s" x="{PAD_X}" y="{y:.1}" font-size="{LEAD_SIZE}" fill="{INK}" letter-spacing=".2">{}</text>"#,
            0.06 + i as f64 * 0.07,
            esc(line)
        ));
    }

    body.push_str(&format!(
        r#"<rect x="{PAD_X}" y="{rule_y:.1}" width="{}" height="1" fill="{LINE}"/>"#,
        W - PAD_X * 2.0
    ));

    if rows > 0 {
        let divider_x = PAD_X + COL_W + COL_GAP / 2.0;
        let divider_y = rule_y + 12.0;
        let divider_h = h - divider_y - 22.0;
        body.push_str(&format!(
            r#"<rect x="{divider_x:.1}" y="{divider_y:.1}" width="1" height="{divider_h:.1}" fill="{LINE}"/>"#
        ));

        for (i, line) in left.iter().enumerate() {
            let y = body_top + i as f64 * BODY_LINE_H;
            body.push_str(&format!(
                r#"<text x="{PAD_X}" y="{y:.1}" font-size="{BODY_SIZE}" fill="{MUTED}" letter-spacing=".1">{}</text>"#,
                esc(line)
            ));
        }

        let right_x = PAD_X + COL_W + COL_GAP;
        for (i, line) in right.iter().enumerate() {
            let y = body_top + i as f64 * BODY_LINE_H;
            body.push_str(&format!(
                r#"<text x="{right_x:.1}" y="{y:.1}" font-size="{BODY_SIZE}" fill="{MUTED}" letter-spacing=".1">{}</text>"#,
                esc(line)
            ));
        }
    }

    doc(W, h, &alt(cfg), &body, "", "", None)
}

/// First sentence becomes the lead; the rest is body copy.
fn split_lead(text: &str) -> (&str, &str) {
    let text = text.trim();
    match text.find(". ") {
        Some(i) => (&text[..=i], text[i + 2..].trim()),
        None => (text, ""),
    }
}

/// Split the body into two roughly equal columns, preferring sentence edges.
fn split_body(text: &str) -> (String, String) {
    let text = text.trim();
    if text.is_empty() {
        return (String::new(), String::new());
    }

    let parts = split_sentences(text);
    if parts.len() >= 2 {
        let total: usize = parts.iter().map(|s| s.chars().count()).sum();
        let mut acc = 0;
        let mut best = 1;
        let mut best_diff = usize::MAX;
        for (i, part) in parts.iter().enumerate().take(parts.len() - 1) {
            acc += part.chars().count();
            let diff = acc.abs_diff(total - acc);
            if diff < best_diff {
                best_diff = diff;
                best = i + 1;
            }
        }
        return (parts[..best].join(" "), parts[best..].join(" "));
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 2 {
        return (text.to_string(), String::new());
    }
    let total = text.chars().count();
    let mut acc = 0;
    let mut best = words.len() / 2;
    let mut best_diff = usize::MAX;
    for (i, word) in words.iter().enumerate().take(words.len() - 1) {
        acc += word.chars().count() + 1;
        let diff = acc.abs_diff(total - acc);
        if diff < best_diff {
            best_diff = diff;
            best = i + 1;
        }
    }
    (words[..best].join(" "), words[best..].join(" "))
}

fn split_sentences(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    for (i, byte) in bytes.iter().enumerate() {
        if matches!(byte, b'.' | b'!' | b'?') && (i + 1 == bytes.len() || bytes[i + 1] == b' ') {
            out.push(text[start..=i].trim().to_string());
            start = i + 1;
        }
    }
    if start < bytes.len() {
        out.push(text[start..].trim().to_string());
    }
    out.retain(|s| !s.is_empty());
    out
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

    fn height(svg: &str) -> f64 {
        svg.split("height=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .parse()
            .unwrap()
    }

    #[test]
    fn it_is_editorial_not_a_terminal() {
        let svg = render(&config("\"one line\""));
        for needle in ["~/about.md", "cat about.md", "class=\"blink\"", "<circle"] {
            assert!(!svg.contains(needle), "{needle} should be gone");
        }
        assert!(svg.contains("01 / ABOUT"));
        assert!(svg.contains("prefers-reduced-motion"));
    }

    #[test]
    fn height_grows_with_the_text() {
        let short = render(&config("\"one line\""));
        let long = render(&config(
            "\"This is a much longer biography that should wrap across several lines and make the panel taller than the short one. It has a second sentence so the two columns have something to balance against each other.\"",
        ));
        assert!(
            height(&short) < height(&long),
            "{} vs {}",
            height(&short),
            height(&long)
        );
    }

    #[test]
    fn very_long_bios_are_capped() {
        let text = "word ".repeat(400);
        let svg = render(&config(&format!("\"{text}\"")));
        assert!(height(&svg) < 400.0, "height {}", height(&svg));
        assert!(svg.contains('…'));
    }
}
