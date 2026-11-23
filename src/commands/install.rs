use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command as Proc;

use anyhow::{Context, Result, anyhow, bail};
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};

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

    let archive_path = std::env::temp_dir().join(format!("rustket-{}.tar.gz", std::process::id()));
    let _archive = TempArchive(archive_path.clone());
    let digest = download(&artifact.url, &archive_path)?;
    if !skip_verify && !digest.eq_ignore_ascii_case(&artifact.sha256) {
        bail!("sha256 mismatch: expected {}, got {digest}", artifact.sha256);
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

/// Stream `url` to `path`, hashing on the fly; returns the hex SHA-256.
fn download(url: &str, path: &Path) -> Result<String> {
    let mut response = Client::new()
        .get(url)
        .send()
        .and_then(|r| r.error_for_status())
        .with_context(|| format!("failed to download {url}"))?;
    let file = fs::File::create(path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    let mut sink = HashWriter(BufWriter::new(file), Sha256::new());
    response
        .copy_to(&mut sink)
        .with_context(|| format!("failed to write {}", path.display()))?;
    sink.0.flush()?;
    Ok(format!("{:x}", sink.1.finalize()))
}

struct HashWriter<W>(W, Sha256);

impl<W: Write> Write for HashWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.0.write(buf)?;
        self.1.update(&buf[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}
