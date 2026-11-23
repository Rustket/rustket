use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::paths;

#[derive(Deserialize)]
pub struct Bit {
    pub name: String,
    pub version: String,
    pub platform: HashMap<String, Artifact>,
}

#[derive(Deserialize)]
pub struct Artifact {
    pub url: String,
    pub sha256: String,
    pub bin: String,
}

pub fn load(name: &str) -> Result<Bit> {
    let matching_files = files()?
        .into_iter()
        .filter(|path| path.file_stem().is_some_and(|stem| stem == name))
        .collect::<Vec<_>>();
    if matching_files.is_empty() {
        bail!("no bit: {name}");
    }
    if matching_files.len() > 1 {
        let locations = matching_files
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        bail!("multiple tapped bits named {name}: {locations}");
    }

    let path = &matching_files[0];
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&contents).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn names() -> Result<Vec<String>> {
    let mut names = files()?
        .into_iter()
        .filter_map(|path| {
            path.file_stem()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    Ok(names)
}

fn files() -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for (_, tap) in paths::tap_dirs(&paths::taps())? {
        for entry in paths::read_dir(&tap.join("bits"))? {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                files.push(path);
            }
        }
    }
    Ok(files)
}
