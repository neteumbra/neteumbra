//! One row of the "select target" list. Rows are separate SVG files (GitHub
//! strips scripts, so every clickable thing needs its own `<img>`), but they
//! share a staggered highlight animation that makes the list feel like a TUI
//! menu with a moving cursor.

use crate::svg::*;

const W: f64 = 900.0;
const DESC_CHARS: usize = 92;
const DESC_LINES: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Repo,
    Open,
    None,
}

#[derive(Debug, Clone)]
pub struct Row<'a> {
    pub index: String,
    pub title: &'a str,
    pub description: &'a str,
    pub tag: &'a str,
    pub stars: Option<u32>,
    pub action: Action,
    pub wip: bool,
}

impl<'a> Row<'a> {
    fn action_label(&self) -> (&'static str, &'static str) {
        if self.wip {
            return ("IN DEV", ACCENT);
        }
        match self.action {
            Action::Repo => ("REPO ↗", MUTED),
            Action::Open => ("OPEN ↗", ACCENT),
            Action::None => ("", MUTED),
        }
    }
}

pub fn render(row: &Row<'_>, idx: usize, total: usize) -> String {
    let lines = wrap(row.description, DESC_CHARS, DESC_LINES);
    let h = 78.0 + (lines.len().saturating_sub(1)) as f64 * 16.0;

    let mut body = String::new();
    body.push_str(&format!(
        r#"<rect x=".5" y=".5" width="{w}" height="{h2}" rx="10" fill="{SURFACE}" stroke="{LINE}"/>"#,
        w = W - 1.0,
        h2 = h - 1.0
    ));

    // Moving highlight: each row lights up once per cycle.
    let cycle = total.max(1) as f64 * 1.6;
    let delay = idx as f64 * 1.6;
    let hold = 72.0 / total.max(1) as f64;
    let fade = 100.0 / total.max(1) as f64;
    body.push_str(&format!(
        r#"<rect class="sel" x="1" y="1" width="{w}" height="{h2}" rx="9" fill="{ACCENT}" opacity="0"/><rect class="edge" x="0" y="12" width="2" height="{eh}" fill="{ACCENT}" opacity="0"/>"#,
        w = W - 2.0,
        h2 = h - 2.0,
        eh = h - 24.0
    ));
    let css = format!(
        r#"
.sel{{animation:sel {cycle:.2}s linear {delay:.2}s infinite}}
.edge{{animation:edge {cycle:.2}s linear {delay:.2}s infinite}}
@keyframes sel{{0%,{hold:.2}%{{opacity:.075}}{fade:.2}%,100%{{opacity:0}}}}
@keyframes edge{{0%,{hold:.2}%{{opacity:.9}}{fade:.2}%,100%{{opacity:0}}}}
"#
    );

    let index = truncate(&row.index, 3);
    body.push_str(&format!(
        r#"<text x="48" y="{y:.1}" font-size="20" font-weight="700" fill="{ACCENT}" letter-spacing=".5">{index}</text><rect x="82" y="18" width="1" height="{dh}" fill="{LINE}"/>"#,
        y = h / 2.0 + 7.0,
        dh = h - 36.0
    ));

    body.push_str(&format!(
        r#"<text x="100" y="33" font-size="17.5" font-weight="700" fill="{INK}" letter-spacing=".4">{}</text>"#,
        esc(&truncate(row.title, 46))
    ));

    for (i, line) in lines.iter().enumerate() {
        body.push_str(&format!(
            r#"<text x="100" y="{y:.1}" font-size="12" fill="{MUTED}" letter-spacing=".1">{}</text>"#,
            esc(line),
            y = 54.0 + i as f64 * 16.0
        ));
    }

    body.push_str(&format!(
        r#"<text x="100" y="{y:.1}" font-size="9.5" fill="{MUTED_2}" letter-spacing="1.3">[ {} ]</text>"#,
        esc(&truncate(&row.tag.to_uppercase(), 22)),
        y = h - 12.0
    ));

    let (label, color) = row.action_label();
    if !label.is_empty() {
        if row.wip {
            body.push_str(&format!(
                r#"<circle cx="771" cy="29" r="3.2" fill="{ACCENT}" class="pulse"/><text x="852" y="33" font-size="10.5" fill="{color}" letter-spacing="1.4" text-anchor="end">{label}</text>"#
            ));
        } else {
            body.push_str(&format!(
                r#"<text x="852" y="33" font-size="10.5" fill="{color}" letter-spacing="1.4" text-anchor="end">{label}</text>"#
            ));
        }
    }

    if let Some(stars) = row.stars {
        body.push_str(&format!(
            r#"<text x="852" y="53" font-size="10.5" fill="{MUTED_2}" letter-spacing="1.2" text-anchor="end">★ {}</text>"#,
            fmt_num(stars)
        ));
    }

    let mut title = format!("{}: {}", row.title, row.description);
    if let Some(stars) = row.stars {
        title.push_str(&format!(" — {stars} stars"));
    }
    if row.wip {
        title.push_str(" (in development)");
    }
    doc(W, h, &title, &body, &css, "", None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(description: &'static str) -> Row<'static> {
        Row {
            index: "01".into(),
            title: "neCyberWiki",
            description,
            tag: "TypeScript",
            stars: Some(9),
            action: Action::Repo,
            wip: false,
        }
    }

    #[test]
    fn short_descriptions_make_short_rows() {
        let svg = render(&row("Short."), 0, 4);
        assert!(svg.contains("height=\"78\""));
        assert!(svg.contains("★ 9"));
        assert!(svg.contains("[ TYPESCRIPT ]"));
    }

    #[test]
    fn long_descriptions_wrap_to_two_lines() {
        let long = "A very long description that definitely needs more than one line to fit inside the row and still look tidy on a profile README, so it keeps going and going until the generator has no choice but to cut it short with an ellipsis at the very end.";
        let svg = render(&row(long), 1, 4);
        assert!(svg.contains("height=\"94\""));
        assert!(svg.contains('…'));
    }

    #[test]
    fn wip_replaces_the_action_label() {
        let mut r = row("Working on it.");
        r.wip = true;
        let svg = render(&r, 0, 2);
        assert!(svg.contains("IN DEV"));
        assert!(!svg.contains("REPO"));
    }

    #[test]
    fn stagger_offsets_each_row() {
        let first = render(&row("A"), 0, 4);
        let second = render(&row("A"), 1, 4);
        assert!(first.contains("linear 0.00s infinite"));
        assert!(second.contains("linear 1.60s infinite"));
        assert!(first.contains("6.40s"));
    }
}
