use anyhow::Result;

use crate::paths;

pub fn run() -> Result<()> {
    for name in installed_names()? {
        println!("{name}");
    }
    Ok(())
}

pub fn installed_names() -> Result<Vec<String>> {
    let mut names = paths::read_dir(&paths::root().join("bin"))?
        .into_iter()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    names.sort();
    Ok(names)
}
