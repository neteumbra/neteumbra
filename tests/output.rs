//! Guard rails for the generated output: well-formed XML, no remote assets, no
//! scripts, and a README whose links resolve to files that actually exist.

use neteumbra_profile::config::Config;
use neteumbra_profile::github::{Cache, RepoMeta, Stats};
use neteumbra_profile::render::build;

fn config() -> Config {
    serde_json::from_str(include_str!("../profile.config.json")).expect("valid config")
}

fn cache() -> Cache {
    let mut cache = Cache::default();
    cache.generated_at = Some("2026-11-13T04:00:00Z".into());
    cache.stats = Stats {
        commits: Some(1_743),
        stars: Some(29),
        public_repos: Some(34),
        followers: Some(14),
    };
    for (repo, desc, lang, stars, home) in [
        (
            "NeTenebrae-Sec/neCyberWiki",
            "Cyber notes gathered while practising: pentesting, CTFs and cheatsheets.",
            "TypeScript",
            9,
            Some("https://netenebrae-sec.github.io/neCyberWiki/"),
        ),
        (
            "NeTenebrae-Sec/neBSPWM",
            "Mi entorno de trabajo en bspwm | EN DESARROLLO",
            "QML",
            10,
            None,
        ),
        (
            "HertexCode/golaya",
            "backend para proyecto principal generation",
            "JavaScript",
            0,
            None,
        ),
        (
            "NeTenebrae-Sec/nenvim",
            "My personal nvim config.",
            "Lua",
            0,
            None,
        ),
    ] {
        cache.repos.insert(
            repo.into(),
            RepoMeta {
                full_name: repo.into(),
                description: Some(desc.into()),
                language: Some(lang.into()),
                stars,
                homepage: home.map(ToString::to_string),
                url: format!("https://github.com/{repo}"),
                updated_at: Some("2026-10-21T21:25:55Z".into()),
            },
        );
    }
    cache
}

#[test]
fn every_svg_is_well_formed_xml_and_self_contained() {
    let output = build(&config(), &cache());
    let mut checked = 0;

    for file in output.files.iter().filter(|f| f.path.ends_with(".svg")) {
        let mut reader = quick_xml::Reader::from_str(&file.content);
        loop {
            match reader.read_event() {
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(_) => {}
                Err(err) => panic!("{} is not valid XML: {err}", file.path),
            }
        }
        assert!(!file.content.contains("<script"), "{} has JS", file.path);
        assert!(
            !file.content.contains("<image"),
            "{} embeds a raster image",
            file.path
        );
        assert!(
            !file.content.contains("href=\"http"),
            "{} references a remote asset",
            file.path
        );
        assert!(
            file.content.contains("role=\"img\"") && file.content.contains("<title>"),
            "{} lacks accessible markup",
            file.path
        );
        assert!(
            file.content.contains("prefers-reduced-motion"),
            "{} lacks reduced-motion handling",
            file.path
        );
        checked += 1;
    }
    assert!(checked >= 15, "only {checked} SVGs checked");
}

#[test]
fn readme_references_only_generated_files() {
    let output = build(&config(), &cache());
    let readme = output
        .files
        .iter()
        .find(|f| f.path == "README.md")
        .expect("README");

    let existing: Vec<&str> = output.files.iter().map(|f| f.path.as_str()).collect();
    let mut found = 0;
    for chunk in readme.content.split("src=\"").skip(1) {
        let src = chunk.split('"').next().unwrap_or_default();
        if src.is_empty() {
            continue;
        }
        assert!(
            existing.contains(&src),
            "README points at missing file: {src}"
        );
        found += 1;
    }
    assert!(found >= 14, "only {found} README images found");
}

#[test]
fn readme_has_accessible_and_plain_text_fallbacks() {
    let output = build(&config(), &cache());
    let readme = output
        .files
        .iter()
        .find(|f| f.path == "README.md")
        .unwrap();

    assert!(readme.content.contains("alt=\"Mauricio — Fullstack Developer."));
    assert!(readme.content.contains("<summary>Plain text version</summary>"));
    assert!(readme.content.contains("cargo test"));
    assert!(readme.content.contains("profile.config.json"));
    assert!(!readme.content.contains("img.shields.io"));
}

#[test]
fn missing_repo_metadata_does_not_break_the_layout() {
    let output = build(&config(), &Cache::default());
    let rows = output.images.targets.len();
    assert_eq!(rows, 4);
    for file in output.files.iter().filter(|f| f.path.contains("target-")) {
        let mut reader = quick_xml::Reader::from_str(&file.content);
        while let Ok(event) = reader.read_event() {
            if matches!(event, quick_xml::events::Event::Eof) {
                break;
            }
        }
    }
}

#[test]
fn output_is_deterministic() {
    let a = build(&config(), &cache());
    let b = build(&config(), &cache());
    let files_a: Vec<(&str, &str)> = a
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.content.as_str()))
        .collect();
    let files_b: Vec<(&str, &str)> = b
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.content.as_str()))
        .collect();
    assert_eq!(files_a, files_b);
}
