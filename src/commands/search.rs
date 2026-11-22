use anyhow::{Context, Result};

use crate::bit;
use crate::commands::tap;

pub fn run(query: &str) -> Result<()> {
    tap::ensure_default()?;
    let query = query.to_lowercase();
    let mut names = bit::names().context("failed to search tapped bits")?;
    names.retain(|name| name.to_lowercase().contains(&query));
    for name in names {
        println!("{name}");
    }
    Ok(())
}
