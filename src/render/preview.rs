//! Local preview page. Not part of the README: open it in a browser to check
//! every piece on GitHub's light and dark surface before pushing.

use crate::svg::esc;

use super::Image;

pub fn render(images: &[&Image]) -> String {
    let mut cards = String::new();
    for image in images {
        cards.push_str(&format!(
            r#"<section>
  <h2>{file}</h2>
  <div class="stage"><img src="{src}" alt="{alt}" /></div>
</section>
"#,
            file = esc(&image.file),
            src = esc(&image.file),
            alt = esc(&image.alt)
        ));
    }

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>neteumbra · stage preview</title>
<style>
  :root {{
    color-scheme: dark;
    --page: #05090a;
    --panel: #0a1113;
    --ink: #f2f6f1;
    --muted: #7e8f86;
    --line: rgba(242,246,241,.12);
    --accent: #7df29a;
  }}
  :root[data-theme="light"] {{
    color-scheme: light;
    --page: #f4f6f4;
    --panel: #ffffff;
    --ink: #101614;
    --muted: #5c6b63;
    --line: rgba(6,16,10,.14);
    --accent: #127a3a;
  }}
  * {{ box-sizing: border-box; }}
  body {{
    margin: 0;
    padding: 32px clamp(16px, 5vw, 64px) 80px;
    background: var(--page);
    color: var(--ink);
    font: 14px/1.6 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  }}
  header {{
    position: sticky; top: 0; z-index: 2;
    display: flex; align-items: center; gap: 16px; flex-wrap: wrap;
    padding: 14px 0; margin-bottom: 8px;
    background: linear-gradient(var(--page) 80%, transparent);
    border-bottom: 1px solid var(--line);
  }}
  header h1 {{ margin: 0; font-size: 15px; letter-spacing: 2px; }}
  header span {{ color: var(--muted); }}
  button {{
    margin-left: auto; cursor: pointer;
    background: transparent; color: var(--accent);
    border: 1px solid var(--accent); border-radius: 6px;
    padding: 6px 12px; font: inherit; letter-spacing: 1px;
  }}
  main {{ max-width: 1000px; margin: 0 auto; }}
  section {{ margin: 34px 0 0; }}
  h2 {{
    margin: 0 0 8px; font-size: 10px; font-weight: 600;
    letter-spacing: 1.6px; color: var(--muted); text-transform: uppercase;
  }}
  .stage {{
    display: flex; justify-content: center;
    padding: 10px; border: 1px dashed var(--line); border-radius: 10px;
  }}
  .stage img {{ max-width: 100%; height: auto; }}
  .note {{ color: var(--muted); margin: 0 0 24px; }}
</style>
</head>
<body>
<header>
  <h1>NETEUMBRA · STAGE</h1>
  <span>generated preview</span>
  <button type="button" id="theme">LIGHT / DARK</button>
</header>
<main>
  <p class="note">Both GitHub themes are simulated here. The SVGs are self-contained: no webfonts, no scripts, no remote assets.</p>
{cards}</main>
<script>
  const root = document.documentElement;
  document.getElementById('theme').addEventListener('click', () => {{
    root.dataset.theme = root.dataset.theme === 'light' ? 'dark' : 'light';
  }});
</script>
</body>
</html>
"#
    )
}
