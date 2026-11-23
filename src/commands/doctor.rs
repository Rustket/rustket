use std::path::Path;
use std::process::Command as Proc;

use anyhow::{Result, anyhow};

use crate::paths;

pub fn run() -> Result<()> {
    let root = paths::root();
    let taps = paths::taps();
    let bin = root.join("bin");
    let mut failures = 0;

    println!("platform: {}", paths::current_platform());
    failures += usize::from(!check("~/.rustket exists", root.is_dir()));
    failures += usize::from(!check("~/.rustket/taps exists", taps.is_dir()));

    let on_path = std::env::var_os("PATH").is_some_and(|path| {
        std::env::split_paths(&path).any(|entry| path_matches_bin(&entry, &bin))
    });
    failures += usize::from(!check("~/.rustket/bin is on PATH", on_path));

    let git_installed = Proc::new("git")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success());
    failures += usize::from(!check("git is installed", git_installed));

    let no_broken_links = !has_broken_symlink(&bin)?;
    failures += usize::from(!check("no broken symlinks in bin", no_broken_links));

    if failures > 0 {
        return Err(anyhow!("{failures} doctor check(s) failed"));
    }
    println!("all good");
    Ok(())
}

fn check(label: &str, passed: bool) -> bool {
    println!("{} - {label}", if passed { "ok" } else { "FAIL" });
    passed
}

fn path_matches_bin(entry: &Path, bin: &Path) -> bool {
    entry == bin
        || matches!(
            (entry.canonicalize(), bin.canonicalize()),
            (Ok(entry), Ok(bin)) if entry == bin
        )
}

fn has_broken_symlink(bin: &Path) -> Result<bool> {
    for entry in paths::read_dir(bin)? {
        let link = entry.path();
        if entry.file_type()?.is_symlink() && !link.exists() {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::path_matches_bin;

    #[test]
    fn missing_paths_do_not_match_when_canonicalization_fails() {
        assert!(!path_matches_bin(
            Path::new("/missing/path-entry"),
            Path::new("/missing/rustket-bin"),
        ));
    }

    #[test]
    fn identical_paths_match_without_canonicalization() {
        let path = Path::new("/missing/rustket-bin");
        assert!(path_matches_bin(path, path));
    }
}
