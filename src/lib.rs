//! neteumbra profile generator.
//!
//! `profile.config.json` is the only file a human edits. This crate turns it,
//! plus the cached GitHub metadata in `assets/profile.json`, into a set of
//! self-contained animated SVGs and the profile README.

pub mod config;
pub mod github;
pub mod render;
pub mod svg;
pub mod time;
