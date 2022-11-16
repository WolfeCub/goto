#![feature(let_chains)]
use std::{fs::File, io, collections::HashMap, process::exit, env};

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
    /// Returns directory path to cd to
    Cd {
        /// Directory bookmark
        key: String,
    },
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
        Commands::Cd { key } => cd(&key),
    };
}

fn cd(key: &str) {
    let mut path = dirs::home_dir().expect("No home dir found");
    path.push(".goto");

    let f = File::open(path.as_path()).expect("Goto file does not exist");

    let goto_file = serde_yaml::from_reader::<_, GotoFile>(io::BufReader::new(f)).expect("Could not deserialize goto file");

    let current_dir = env::current_dir().expect("Process has no working directory");

    let alias_directory = goto_file.projects.iter().find_map(|project| {
        if current_dir.starts_with(&project.root) && let Some(value) = project.aliases.get(key) {
            Some(value)
        } else {
            None
        }
    });

    if let Some(value) = alias_directory {
        print!("{}", *value);
    } else {
        eprintln!("Alias '{}' was not found in any projects", key);
        exit(2);
    }
}
