use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command as Proc;

use anyhow::{Context, Result, anyhow, bail};
use reqwest::blocking::Client;

use crate::bit;
use crate::commands::tap;
use crate::paths;

struct TempArchive(PathBuf);

impl Drop for TempArchive {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub fn run(name: &str, skip_verify: bool) -> Result<()> {
    tap::ensure_default()?;
    let b = bit::load(name)?;
    let platform = paths::current_platform();
    let artifact = b
        .platform
        .get(&platform)
        .ok_or_else(|| anyhow!("{} has no build for {}", b.name, platform))?;

    let response = Client::new()
        .get(&artifact.url)
        .send()
        .with_context(|| format!("failed to download {}", artifact.url))?
        .error_for_status()
        .with_context(|| format!("download failed for {}", artifact.url))?;
    let archive_path = std::env::temp_dir().join(format!("rustket-{}.tar.gz", std::process::id()));
    let _archive = TempArchive(archive_path.clone());
    let mut archive = fs::File::create(&archive_path)
        .with_context(|| format!("failed to create {}", archive_path.display()))?;
    archive
        .write_all(&response.bytes()?)
        .with_context(|| format!("failed to write {}", archive_path.display()))?;

    if !skip_verify {
        verify_sha256(&archive_path, &artifact.sha256)?;
    }

    let cellar = paths::root().join("cellar").join(&b.name).join(&b.version);
    fs::create_dir_all(&cellar)
        .with_context(|| format!("failed to create {}", cellar.display()))?;
    let output = Proc::new("tar")
        .args(["-xzf"])
        .arg(&archive_path)
        .arg("-C")
        .arg(&cellar)
        .output()
        .context("failed to run tar")?;
    if !output.status.success() {
        bail!(
            "failed to unpack archive: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let binary = fs::canonicalize(cellar.join(&artifact.bin))
        .with_context(|| format!("artifact binary not found: {}", artifact.bin))?;
    let bin_dir = paths::root().join("bin");
    fs::create_dir_all(&bin_dir)
        .with_context(|| format!("failed to create {}", bin_dir.display()))?;
    let link = bin_dir.join(name);
    if let Ok(metadata) = fs::symlink_metadata(&link) {
        if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
            bail!("cannot replace directory at {}", link.display());
        }
        fs::remove_file(&link).with_context(|| format!("failed to remove {}", link.display()))?;
    }
    create_symlink(&binary, &link)
        .with_context(|| format!("failed to link {} to {}", link.display(), binary.display()))?;
    println!("installed {} {}", b.name, b.version);
    Ok(())
}

fn verify_sha256(path: &std::path::Path, expected: &str) -> Result<()> {
    let output = match Proc::new("shasum").args(["-a", "256"]).arg(path).output() {
        Ok(output) if output.status.success() => output,
        _ => Proc::new("sha256sum")
            .arg(path)
            .output()
            .context("failed to run shasum or sha256sum")?,
    };
    if !output.status.success() {
        bail!(
            "sha256 verification failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = stdout
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("sha256 tool returned no digest"))?;
    if !actual.eq_ignore_ascii_case(expected) {
        bail!("sha256 mismatch: expected {expected}, got {actual}");
    }
    Ok(())
}

#[cfg(unix)]
fn create_symlink(target: &std::path::Path, link: &std::path::Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_symlink(target: &std::path::Path, link: &std::path::Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}
