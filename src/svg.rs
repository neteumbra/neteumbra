//! SVG primitives shared by every piece of the stage.
//!
//! GitHub serves SVGs inside `<img>`: no scripts, no external fonts, no network
//! requests. Everything is therefore inline, monospace (system font stack) and
//! self-contained. Animations are pure CSS and are disabled when the reader asks
//! for reduced motion.

pub const BG: &str = "#05090a";
pub const BG_DEEP: &str = "#020405";
pub const SURFACE: &str = "#0a1113";
pub const SURFACE_2: &str = "#0e1719";
pub const SURFACE_3: &str = "#132124";
pub const INK: &str = "#f2f6f1";
pub const INK_DARK: &str = "#06100a";
pub const MUTED: &str = "#a8b6ae";
pub const MUTED_2: &str = "#7e8f86";
pub const ACCENT: &str = "#7df29a";
pub const ACCENT_STRONG: &str = "#a8ffbc";
pub const LINE: &str = "rgba(242,246,241,0.10)";
pub const LINE_STRONG: &str = "rgba(242,246,241,0.20)";

pub const MONO: &str = "'JetBrains Mono','Fira Code','SFMono-Regular',Menlo,Consolas,'DejaVu Sans Mono',monospace";
/// Advance width of the system monospace stack, as a fraction of the font size.
const CHAR_ADVANCE: f64 = 0.601;

pub fn esc(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

pub fn text_width(text: &str, size: f64, letter_spacing: f64) -> f64 {
    text.chars().count() as f64 * (size * CHAR_ADVANCE + letter_spacing)
}

pub fn fit_chars(max_width: f64, size: f64, letter_spacing: f64) -> usize {
    let adv = size * CHAR_ADVANCE + letter_spacing;
    (max_width / adv).floor().max(1.0) as usize
}

pub fn truncate(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    let keep = max_chars.saturating_sub(1).max(1);
    let mut out: String = text.chars().take(keep).collect();
    out.push('…');
    out
}

/// Greedy wrap with a hard line budget; the final visible line gets an ellipsis
/// when content is cut. Widths are character-based because the font is
/// monospace, which keeps the layout deterministic across platforms.
pub fn wrap(text: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in normalized.split(' ') {
        if word.is_empty() {
            continue;
        }
        let candidate_len = if current.is_empty() {
            word.chars().count()
        } else {
            current.chars().count() + 1 + word.chars().count()
        };
        if candidate_len <= max_chars || current.is_empty() {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(&truncate(word, max_chars));
        } else {
            lines.push(std::mem::take(&mut current));
            if lines.len() == max_lines {
                break;
            }
            current.push_str(&truncate(word, max_chars));
        }
    }
    if !current.is_empty() && lines.len() < max_lines {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }

    let consumed: usize = lines.iter().map(|l| l.chars().count()).sum::<usize>()
        + lines.len().saturating_sub(1);
    let total = normalized.chars().count();
    if lines.len() == max_lines && consumed < total {
        let last = lines.last_mut().expect("non-empty");
        if last.chars().count() >= max_chars {
            *last = truncate(last, max_chars);
        } else {
            last.push('…');
        }
    }
    lines
}

pub fn fmt_num(value: u32) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

pub fn base_css() -> String {
    format!(
        r#"
text{{font-family:{MONO};}}
.blink{{animation:blink 1.15s steps(1) infinite}}
@keyframes blink{{0%,49%{{opacity:1}}50%,100%{{opacity:0}}}}
.pulse{{animation:pulse 2.6s ease-in-out infinite}}
@keyframes pulse{{0%,100%{{opacity:1}}50%{{opacity:.35}}}}
.rise{{animation:rise .55s cubic-bezier(.22,1,.36,1) both}}
@keyframes rise{{from{{opacity:0;transform:translateY(9px)}}to{{opacity:1;transform:none}}}}
.slam{{animation:slam .5s cubic-bezier(.22,1,.36,1) both}}
@keyframes slam{{from{{opacity:0;transform:translateX(-26px)}}to{{opacity:1;transform:none}}}}
.mq{{animation:mq var(--mq-dur) linear infinite}}
@keyframes mq{{to{{transform:translateX(var(--mq-shift))}}}}
@media (prefers-reduced-motion:reduce){{*{{animation:none!important;transition:none!important}}}}
"#
    )
}

/// Full document. `round` clips the body to a rounded frame, like a panel.
pub fn doc(w: f64, h: f64, title: &str, body: &str, css: &str, defs: &str, round: Option<f64>) -> String {
    let (defs, body) = match round {
        Some(r) => (
            format!(
                r#"<clipPath id="frame"><rect width="{w}" height="{h}" rx="{r}"/></clipPath>{defs}"#
            ),
            format!(r#"<g clip-path="url(#frame)">{body}</g>"#),
        ),
        None => (defs.to_string(), body.to_string()),
    };
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" role="img" aria-label="{label}"><title>{label}</title><style>{base}{css}</style><defs>{defs}</defs>{body}</svg>
"#,
        label = esc(title),
        base = base_css(),
    )
}

pub fn corner_brackets(w: f64, h: f64, inset: f64, len: f64) -> String {
    let x0 = inset;
    let y0 = inset;
    let x1 = w - inset;
    let y1 = h - inset;
    let stroke = format!(
        r#"fill="none" stroke="{ACCENT}" stroke-opacity=".34" stroke-width="1.4""#
    );
    format!(
        r#"<path d="M{x0} {y0b}V{y0}H{x0b}" {stroke}/><path d="M{x1a} {y0}H{x1}V{y0b}" {stroke}/><path d="M{x0} {y1a}V{y1}H{x0b}" {stroke}/><path d="M{x1a} {y1}H{x1}V{y1a}" {stroke}/>"#,
        y0b = y0 + len,
        x0b = x0 + len,
        x1a = x1 - len,
        y1a = y1 - len,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_markup() {
        assert_eq!(esc(r#"<a & "b">"#), "&lt;a &amp; &quot;b&quot;&gt;");
    }

    #[test]
    fn wraps_within_the_budget() {
        let lines = wrap("one two three four five six", 9, 3);
        assert!(lines.len() <= 3);
        assert!(lines.iter().all(|l| l.chars().count() <= 10));
    }

    #[test]
    fn adds_an_ellipsis_when_cutting_content() {
        let lines = wrap("alpha beta gamma delta epsilon zeta eta theta", 10, 2);
        assert_eq!(lines.len(), 2);
        assert!(lines.last().unwrap().ends_with('…'), "{lines:?}");
    }

    #[test]
    fn short_text_is_untouched() {
        assert_eq!(wrap("hello", 20, 2), vec!["hello"]);
    }

    #[test]
    fn empty_text_still_has_one_line() {
        assert_eq!(wrap("   ", 20, 2), vec![""]);
    }

    #[test]
    fn truncates_unicode_by_chars() {
        assert_eq!(truncate("abcdef", 4), "abc…");
        assert_eq!(truncate("ab", 4), "ab");
    }

    #[test]
    fn formats_thousands() {
        assert_eq!(fmt_num(0), "0");
        assert_eq!(fmt_num(999), "999");
        assert_eq!(fmt_num(1_234), "1,234");
        assert_eq!(fmt_num(1_234_567), "1,234,567");
    }

    #[test]
    fn fitted_text_never_exceeds_the_width() {
        let max = 300.0;
        let chars = fit_chars(max, 12.0, 0.5);
        let sample = "x".repeat(chars);
        assert!(text_width(&sample, 12.0, 0.5) <= max);
    }
}
