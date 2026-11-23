use std::fs::{self, DirEntry};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub fn root() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .expect("no home dir")
        .join(".rustket")
}

pub fn taps() -> PathBuf {
    root().join("taps")
}

pub fn current_platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Read a directory; a missing directory yields no entries.
pub fn read_dir(path: &Path) -> Result<Vec<DirEntry>> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };
    entries
        .collect::<std::io::Result<_>>()
        .with_context(|| format!("failed to read {}", path.display()))
}

/// Subdirectories (not symlinks) of `path`.
fn subdirs(path: &Path) -> Result<Vec<DirEntry>> {
    let mut dirs = Vec::new();
    for entry in read_dir(path)? {
        let kind = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?;
        if kind.is_dir() {
            dirs.push(entry);
        }
    }
    Ok(dirs)
}

/// All `owner/repo` tap checkouts under `tap_root`, sorted by name.
pub fn tap_dirs(tap_root: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut taps = Vec::new();
    for owner in subdirs(tap_root)? {
        for repo in subdirs(&owner.path())? {
            let name = format!(
                "{}/{}",
                owner.file_name().to_string_lossy(),
                repo.file_name().to_string_lossy()
            );
            taps.push((name, repo.path()));
        }
    }
    taps.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(taps)
}
