//! Section header: HUD label, oversized ghost word, hairline rule.

use crate::svg::*;

const W: f64 = 900.0;
const H: f64 = 72.0;

pub fn render(title: &str, meta: &str) -> String {
    let mut body = String::new();
    body.push_str(&format!(
        r#"<rect width="{W}" height="{H}" fill="{BG}"/><text x="852" y="56" font-size="52" font-weight="800" fill="{INK}" fill-opacity=".05" letter-spacing="2" text-anchor="end">{}</text>"#,
        esc(&title.to_uppercase())
    ));
    body.push_str(&format!(
        r#"<g class="slam"><text x="48" y="44" font-size="16" font-weight="700" fill="{ACCENT}" letter-spacing="1">▸▸</text><text x="86" y="44" font-size="19" font-weight="700" fill="{INK}" letter-spacing="3.4">{}</text><text x="852" y="44" font-size="9.5" fill="{MUTED_2}" letter-spacing="1.6" text-anchor="end">{}</text></g>"#,
        esc(&title.to_uppercase()),
        esc(meta)
    ));
    body.push_str(&format!(
        r#"<rect x="48" y="62" width="804" height="1" fill="{LINE}"/><rect x="48" y="62" width="54" height="1" fill="{ACCENT}"/>"#
    ));
    doc(W, H, title, &body, "", "", None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uppercase_title_and_meta_render() {
        let svg = render("Select target", "[ 04 PROJECTS ]");
        assert!(svg.contains("SELECT TARGET"));
        assert!(svg.contains("[ 04 PROJECTS ]"));
        assert!(svg.contains("viewBox=\"0 0 900 72\""));
    }
}
