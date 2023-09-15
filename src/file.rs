use std::{collections::HashMap, env, fs::File, io, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GotoFile {
    pub projects: Vec<Project>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub root: String,
    pub aliases: HashMap<String, String>,
}

fn get_goto_path() -> anyhow::Result<PathBuf> {
    let mut path = dirs::home_dir().with_context(|| "No home dir found")?;
    path.push(".goto.yaml");

    Ok(path)
}

pub fn deserialize_goto_file() -> anyhow::Result<GotoFile> {
    let path = get_goto_path()?;
    let f = File::open(path.as_path()).with_context(|| "File `{path}` does not exist")?;

    serde_yaml::from_reader::<_, GotoFile>(io::BufReader::new(f))
        .with_context(|| "Could not deserialize file `{path}`")
}

fn serialize_goto_file(goto_file: GotoFile) -> anyhow::Result<()> {
    let path = get_goto_path()?;
    let f = File::create(path.as_path()).with_context(|| "File `{path}` does not exist")?;

    serde_yaml::to_writer(io::BufWriter::new(f), &goto_file)
        .with_context(|| "Failed to update file: `{path}`")?;
    Ok(())
}

pub fn get_project_for_current_dir() -> anyhow::Result<Option<Project>> {
    let current_dir = env::current_dir().with_context(|| "Process has no working directory")?;
    let goto_file = deserialize_goto_file()?;

    Ok(goto_file
        .projects
        .into_iter()
        .find(|project| current_dir.starts_with(&project.root)))
}
