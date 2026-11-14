use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use neteumbra_profile::config::Config;
use neteumbra_profile::github::{self, Cache};
use neteumbra_profile::render;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    let mode = args.first().map(String::as_str).unwrap_or("all");
    let check = args.iter().any(|a| a == "--check");
    let cfg = Config::load(&root.join("profile.config.json"))?;
    let cache_path = root.join("assets/profile.json");

    let cache = if check {
        Cache::load(&cache_path)
    } else {
        match mode {
            "fetch" => {
                github::fetch(&cfg, &cache_path)?;
                println!(
                    "· cache refreshed; run `cargo run --release -- generate` to rebuild the stage"
                );
                return Ok(());
            }
            "generate" | "offline" => Cache::load(&cache_path),
            "all" => match github::fetch(&cfg, &cache_path) {
                Ok(cache) => cache,
                Err(err) => {
                    eprintln!("warn: GitHub refresh failed, using cached data: {err:#}");
                    Cache::load(&cache_path)
                }
            },
            other => return Err(anyhow!("unknown command `{other}` (try --help)")),
        }
    };

    let output = render::build(&cfg, &cache);

    if check {
        let mut stale = Vec::new();
        for file in &output.files {
            let path = root.join(&file.path);
            if read(&path).as_deref() != Some(file.content.as_str()) {
                stale.push(file.path.clone());
            }
        }
        if !stale.is_empty() {
            for path in &stale {
                eprintln!("stale: {path}");
            }
            return Err(anyhow!(
                "{} generated file(s) out of date — run `cargo run --release`",
                stale.len()
            ));
        }
        println!("✓ {} generated files are up to date", output.files.len());
        return Ok(());
    }

    let mut written = 0usize;
    let mut unchanged = 0usize;
    for file in &output.files {
        let path = root.join(&file.path);
        if write_if_changed(&path, &file.content)? {
            written += 1;
            println!("  ✓ {}", file.path);
        } else {
            unchanged += 1;
        }
    }
    println!("· {written} written, {unchanged} unchanged");
    Ok(())
}

fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    if read(path).as_deref() == Some(content) {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("cannot create {}", parent.display()))?;
    }
    std::fs::write(path, content).with_context(|| format!("cannot write {}", path.display()))?;
    Ok(true)
}

fn read(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

fn print_help() {
    println!(
        "neteumbra profile generator

USAGE
    cargo run --release                refresh GitHub data + regenerate everything
    cargo run --release -- fetch       refresh assets/profile.json only
    cargo run --release -- generate    regenerate from the cached data (offline)
    cargo run --release -- --check     fail if the generated files are stale
    cargo run --release -- --help      show this help

The only file you edit by hand is profile.config.json. Everything under
assets/stage/, README.md and preview.html is generated."
    );
}
