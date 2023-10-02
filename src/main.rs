#![feature(let_chains)]
use anyhow::{Context, Result};
use std::{fs, env};

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
        wrapper.add_option(NicknamedDir::new(nickname, &path, ""))?;

        for file in fs::read_dir(&path)
            .with_context(|| format!("Unable to read directory: {}", path))?
            .filter(|f| f.as_ref().expect("Unable to read file").path().is_dir())
            .filter_map(|f| f.ok())
        {
            let name = file
                .file_name()
                .into_string()
                .expect("Unable to read file name");
            wrapper.add_option(NicknamedDir::new(nickname, &path, &name))?;
        }
    }

    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = wrapper.run(args.join(" ").as_str())?;

    println!("{}", result.path);
    Ok(())
}
