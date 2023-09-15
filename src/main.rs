#![feature(let_chains)]
use anyhow::{Context, Result};
use std::fs;

mod fuzzy;
use fuzzy::*;

mod file;
use file::*;

pub fn main() -> Result<()> {
    let wrapper = FuzzyMenu::<NicknamedDir>::new();

    let project = get_project_for_current_dir()
        .transpose()
        .unwrap_or_else(|| fuzzy_select_project())?;

    for (nickname, full_path) in project.aliases.iter() {
        let path = format!("{}/{full_path}", project.root);
        for file in fs::read_dir(&path)
            .with_context(|| format!("Unable to read directory: {}", path))?
            .filter(|f| f.as_ref().expect("Unable to read file").path().is_dir())
            .filter_map(|f| f.ok())
        {
            let name = file
                .file_name()
                .into_string()
                .expect("Unable to read file name");
            wrapper.add_option(NicknamedDir::new(nickname, &name, &path))?;
        }
    }

    let result = wrapper.run()?;

    println!("{}", result.path);
    Ok(())
}
