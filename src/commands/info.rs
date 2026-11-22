use anyhow::Result;

use crate::bit;
use crate::commands::tap;

pub fn run(name: &str) -> Result<()> {
    tap::ensure_default()?;
    let b = bit::load(name)?;
    let mut platforms = b.platform.keys().collect::<Vec<_>>();
    platforms.sort();

    println!("name: {}", b.name);
    println!("version: {}", b.version);
    println!("platforms:");
    for platform in platforms {
        println!("  {platform}");
    }
    Ok(())
}
