//! Turns config + cache into files on disk.

pub mod about;
pub mod contact;
pub mod hero;
pub mod preview;
pub mod readme;
pub mod row;
pub mod section;
pub mod ticker;

use crate::config::{slug, Config};
use crate::github::Cache;

use row::{Action, Row};

#[derive(Debug, Clone, Default)]
pub struct Image {
    pub file: String,
    pub alt: String,
    pub href: Option<String>,
    /// `None` means "full width"; `Some` is a fixed height for inline chips.
    pub height: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct Images {
    pub hero: Image,
    pub about: Image,
    pub target_header: Image,
    pub targets: Vec<Image>,
    pub demos: Vec<Image>,
    pub signal_header: Option<Image>,
    pub signals: Vec<Image>,
    pub contact_header: Image,
    pub contacts: Vec<Image>,
    pub ticker: Image,
}

impl Images {
    pub fn all(&self) -> Vec<&Image> {
        let mut out = vec![&self.hero, &self.about, &self.target_header];
        out.extend(self.targets.iter());
        out.extend(self.demos.iter());
        if let Some(header) = &self.signal_header {
            out.push(header);
        }
        out.extend(self.signals.iter());
        out.push(&self.contact_header);
        out.extend(self.contacts.iter());
        out.push(&self.ticker);
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub path: String,
    pub content: String,
}

pub struct Output {
    pub files: Vec<File>,
    pub images: Images,
}

pub fn build(cfg: &Config, cache: &Cache) -> Output {
    let projects = cfg.projects(cache);
    let signals = cfg.signals();
    let mut files: Vec<File> = Vec::new();

    let mut images = Images {
        hero: image(
            "assets/stage/hero.svg",
            hero::alt(cfg),
            None,
            None,
            &mut files,
            hero::render(cfg, cache),
        ),
        about: image(
            "assets/stage/about.svg",
            cfg.about.clone(),
            None,
            None,
            &mut files,
            about::render(cfg),
        ),
        target_header: image(
            "assets/stage/h-targets.svg",
            "Select target: featured projects",
            None,
            None,
            &mut files,
            section::render("Select target", &format!("[ {:02} PROJECTS ]", projects.len())),
        ),
        ..Default::default()
    };

    for (i, project) in projects.iter().enumerate() {
        let action = if project.repo_url.is_some() {
            Action::Repo
        } else if project.live_url.is_some() {
            Action::Open
        } else {
            Action::None
        };
        let row = Row {
            index: format!("{:02}", i + 1),
            title: &project.label,
            description: &project.description,
            tag: &project.tag,
            stars: project.stars,
            action,
            wip: project.wip,
        };
        let file = format!("assets/stage/target-{i}.svg");
        let mut alt = format!(
            "{:02} — {}: {}",
            i + 1,
            project.label,
            project.description
        );
        if let Some(stars) = project.stars {
            alt.push_str(&format!(" ({stars} stars)"));
        }
        if project.wip {
            alt.push_str(" — in development");
        }
        images.targets.push(image(
            &file,
            alt,
            project.primary_url().map(ToOwned::to_owned),
            None,
            &mut files,
            row::render(&row, i, projects.len()),
        ));
    }

    let mut demo_index = 0usize;
    for project in &projects {
        let Some(live) = &project.live_url else { continue };
        let (_, svg) = row::live_chip(&project.label);
        let file = format!("assets/stage/demo-{demo_index}.svg");
        demo_index += 1;
        images.demos.push(image(
            &file,
            format!("Live demo: {}", project.label),
            Some(live.clone()),
            Some(32.0),
            &mut files,
            svg,
        ));
    }

    if !signals.is_empty() {
        images.signal_header = Some(image(
            "assets/stage/h-signal.svg",
            "Signal: elsewhere on the internet",
            None,
            None,
            &mut files,
            section::render("Signal", &format!("[ {:02} SIGNALS ]", signals.len())),
        ));
        for (i, signal) in signals.iter().enumerate() {
            let description = if signal.description.trim().is_empty() {
                signal.value.clone()
            } else {
                format!("{} · {}", signal.value, signal.description)
            };
            let row = Row {
                index: "▸".into(),
                title: &signal.title,
                description: &description,
                tag: &signal.label,
                stars: None,
                action: if signal.url.is_some() {
                    Action::Open
                } else {
                    Action::None
                },
                wip: false,
            };
            let file = format!("assets/stage/signal-{i}.svg");
            images.signals.push(image(
                &file,
                format!("{} — {}", signal.title, description),
                signal.url.clone(),
                None,
                &mut files,
                row::render(&row, i, signals.len()),
            ));
        }
    }

    images.contact_header = image(
        "assets/stage/h-contact.svg",
        "Contact",
        None,
        None,
        &mut files,
        section::render("Contact", &format!("[ {:02} LINKS ]", cfg.contacts.len())),
    );
    for contact in &cfg.contacts {
        let file = format!("assets/stage/contact-{}.svg", slug(&contact.label));
        images.contacts.push(image(
            &file,
            format!("{} ↗", contact.label),
            Some(contact.url.clone()),
            Some(44.0),
            &mut files,
            contact::render(&contact.label, contact.filled),
        ));
    }

    let (ticker_svg, ticker_alt) = ticker::render(cfg, cache);
    images.ticker = image(
        "assets/stage/ticker.svg",
        ticker_alt,
        None,
        None,
        &mut files,
        ticker_svg,
    );

    let readme = readme::render(&readme::Inputs {
        cfg,
        cache,
        projects: &projects,
        images: &images,
    });
    files.push(File {
        path: "README.md".into(),
        content: readme,
    });

    let preview = preview::render(&images.all());
    files.push(File {
        path: "preview.html".into(),
        content: preview,
    });

    Output { files, images }
}

#[allow(clippy::too_many_arguments)]
fn image(
    file: &str,
    alt: impl Into<String>,
    href: Option<String>,
    height: Option<f64>,
    files: &mut Vec<File>,
    content: String,
) -> Image {
    files.push(File {
        path: file.to_string(),
        content,
    });
    Image {
        file: file.to_string(),
        alt: alt.into(),
        href,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::{RepoMeta, Stats};

    fn config() -> Config {
        serde_json::from_str(include_str!("../../profile.config.json")).unwrap()
    }

    fn cache() -> Cache {
        let mut cache = Cache::default();
        cache.generated_at = Some("2026-11-13T00:00:00Z".into());
        cache.stats = Stats {
            commits: Some(120),
            stars: Some(29),
            public_repos: Some(30),
            followers: Some(14),
        };
        cache.repos.insert(
            "NeTenebrae-Sec/neCyberWiki".into(),
            RepoMeta {
                full_name: "NeTenebrae-Sec/neCyberWiki".into(),
                description: Some(
                    "Cyber notes gathered while practising: pentesting, CTFs and cheatsheets."
                        .into(),
                ),
                language: Some("TypeScript".into()),
                stars: 9,
                homepage: Some("https://netenebrae-sec.github.io/neCyberWiki/".into()),
                url: "https://github.com/NeTenebrae-Sec/neCyberWiki".into(),
                updated_at: Some("2026-10-21T21:25:55Z".into()),
            },
        );
        cache
    }

    #[test]
    fn build_is_deterministic() {
        let a = build(&config(), &cache());
        let b = build(&config(), &cache());
        assert_eq!(a.files, b.files);
    }

    #[test]
    fn expected_files_exist() {
        let out = build(&config(), &cache());
        let paths: Vec<&str> = out.files.iter().map(|f| f.path.as_str()).collect();
        for expected in [
            "README.md",
            "preview.html",
            "assets/stage/hero.svg",
            "assets/stage/about.svg",
            "assets/stage/h-targets.svg",
            "assets/stage/target-0.svg",
            "assets/stage/demo-0.svg",
            "assets/stage/h-signal.svg",
            "assets/stage/signal-0.svg",
            "assets/stage/h-contact.svg",
            "assets/stage/contact-github.svg",
            "assets/stage/ticker.svg",
        ] {
            assert!(paths.contains(&expected), "missing {expected}");
        }
        assert_eq!(out.images.targets.len(), 4);
    }

    #[test]
    fn readme_references_every_image_and_nothing_remote() {
        let out = build(&config(), &cache());
        let readme = out
            .files
            .iter()
            .find(|f| f.path == "README.md")
            .unwrap()
            .content
            .clone();
        for image in out.images.all() {
            assert!(
                readme.contains(&format!("src=\"{}\"", image.file)),
                "README does not reference {}",
                image.file
            );
        }
        assert!(!readme.contains("img.shields.io"));
        assert!(!readme.contains("user-attachments"));
        assert!(!readme.contains("<script"));
        assert!(readme.contains("Plain text version"));
        assert!(readme.contains("How this README is built"));
    }

    #[test]
    fn preview_points_at_the_real_files() {
        let out = build(&config(), &cache());
        let preview = out
            .files
            .iter()
            .find(|f| f.path == "preview.html")
            .unwrap()
            .content
            .clone();
        assert!(preview.contains("src=\"assets/stage/hero.svg\""));
        assert!(!preview.contains("src=\"../"));
    }

    #[test]
    fn empty_cache_still_renders() {
        let out = build(&config(), &Cache::default());
        let hero = out
            .files
            .iter()
            .find(|f| f.path == "assets/stage/hero.svg")
            .unwrap();
        assert!(hero.content.contains("MAURICIO"));
        assert!(hero.content.contains("AVAILABLE FOR NEW PROJECTS"));
        let ticker = out
            .files
            .iter()
            .find(|f| f.path == "assets/stage/ticker.svg")
            .unwrap();
        assert!(!ticker.content.contains("COMMITS"));
    }
}
