#![feature(let_chains)]
use std::{fs::File, io, collections::HashMap, process::exit, env, path::PathBuf};

use clap::{Parser, command, Subcommand};
use serde::{Deserialize, Serialize};

/// Simple program to bookmark directories
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Returns directory path corresponding to the provided alias
    Cd {
        /// Directory alias
        key: String,
    },
    Add {
        project: String,
        alias: String,
        directory: String,
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GotoFile {
    projects: Vec<Project>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Project {
    root: String,
    aliases: HashMap<String, String>,
}

fn main() {
    let args = Args::parse();


    match args.command {
        Commands::Cd { key: alias } => cd_cmd(&alias),
        Commands::Add { project, alias, directory } => add_cmd(project, alias, directory),
    };
}

fn get_goto_path() -> PathBuf {
    let mut path = dirs::home_dir().expect("No home dir found");
    path.push(".goto.yaml");

    path
}

fn deserialize_goto_file() -> GotoFile {
    let path = get_goto_path();
    let f = File::open(path.as_path()).expect("Goto file does not exist");

    serde_yaml::from_reader::<_, GotoFile>(io::BufReader::new(f)).expect("Could not deserialize goto file")
}

fn serialize_goto_file(goto_file: GotoFile) {
    let path = get_goto_path();
    let f = File::create(path.as_path()).expect("Goto file does not exist");

    serde_yaml::to_writer(io::BufWriter::new(f), &goto_file).expect("Failed to update goto file");
}

fn cd_cmd(key: &str) {
    let current_dir = env::current_dir().expect("Process has no working directory");
    let goto_file = deserialize_goto_file();

    let alias_directory = goto_file.projects.iter().find_map(|project| {
        if current_dir.starts_with(&project.root) && let Some(value) = project.aliases.get(key) {
            let path: PathBuf = [&project.root, value].iter().collect();
            Some(path)
        } else {
            None
        }
    });

    if let Some(value) = alias_directory {
        print!("{}", value.to_str().expect("Path does not form a valid string"));
    } else {
        eprintln!("Alias '{}' was not found in any projects", key);
        exit(2);
    }
}

fn add_cmd(project: String, alias: String, directory: String) {
    let mut goto_file = deserialize_goto_file();

    if let Some(proj) = goto_file.projects.iter_mut().find(|p| p.root == project) {
        proj.aliases.insert(alias, directory);
    } else {
        let proj = Project {
            root: project,
            aliases: HashMap::from_iter([(alias, directory)]),
        };
        goto_file.projects.push(proj);
    }
    serialize_goto_file(goto_file);
}
