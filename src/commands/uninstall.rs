use std::fs;
use std::io::{ErrorKind, Result as IoResult};
use std::path::Path;

use anyhow::{Context, Result};

use crate::paths;

pub fn run(name: &str) -> Result<()> {
    let root = paths::root();
    let link = root.join("bin").join(name);
    ignore_missing(fs::remove_file(&link), &link)?;
    let cellar = root.join("cellar").join(name);
    ignore_missing(fs::remove_dir_all(&cellar), &cellar)?;
    println!("uninstalled {name}");
    Ok(())
}

fn ignore_missing(result: IoResult<()>, path: &Path) -> Result<()> {
    match result {
        Err(e) if e.kind() != ErrorKind::NotFound => {
            Err(e).with_context(|| format!("failed to remove {}", path.display()))
        }
        _ => Ok(()),
    }
}
