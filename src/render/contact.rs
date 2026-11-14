//! Contact chips. Each chip is its own SVG wrapped in a link, so they wrap
//! naturally on narrow screens.

use crate::svg::*;

const W: f64 = 150.0;
const H: f64 = 44.0;

pub fn render(label: &str, filled: bool) -> String {
    let (bg, fg, border) = if filled {
        (ACCENT, INK_DARK, ACCENT)
    } else {
        (SURFACE, MUTED, LINE_STRONG)
    };
    let text = format!("{} ↗", label.to_uppercase());
    let text = truncate(&text, 18);
    let mut body = String::new();
    body.push_str(&format!(
        r#"<rect x=".5" y=".5" width="{w}" height="{h}" rx="8" fill="{bg}" stroke="{border}"/><text x="{cx}" y="27" font-size="10.5" fill="{fg}" letter-spacing="1.8" text-anchor="middle">{text}</text>"#,
        w = W - 1.0,
        h = H - 1.0,
        cx = W / 2.0,
    ));
    doc(W, H, label, &body, "", "", None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filled_and_outline_variants() {
        let filled = render("GITHUB", true);
        let outline = render("LINKEDIN", false);
        assert!(filled.contains("GITHUB ↗"));
        assert!(outline.contains("LINKEDIN ↗"));
        assert!(filled.contains(ACCENT));
        assert!(outline.contains(SURFACE));
    }

    #[test]
    fn long_labels_are_truncated() {
        let svg = render("A VERY LONG NETWORK NAME", false);
        assert!(svg.contains('…'));
    }
}
