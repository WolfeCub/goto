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
        alias: String,
        directory: String,

        /// Project to add alias to. Defaults to current project
        #[arg(short, long)]
        project: Option<String>,
    },
    Ls {
        /// Show all project aliases
        #[arg(long)]
        all: bool,
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
        Commands::Add { alias, directory, project } => add_cmd(alias, directory, project),
        Commands::Ls { all } => ls_cmd(all),
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

fn add_cmd(alias: String, directory: String, project: Option<String>) {
    let mut goto_file = deserialize_goto_file();
    let current_dir = env::current_dir().expect("Process has no working directory");
    let directory = directory.trim_end_matches('/');

    if let Some(proj) = project {
        let proj = Project {
            root: proj,
            aliases: HashMap::from_iter([(alias, directory.to_owned())]),
        };
        goto_file.projects.push(proj);
    } else if let Some(current_project) = goto_file.projects.iter_mut().find(|p| current_dir.starts_with(&p.root)) {
        current_project.aliases.insert(alias, directory.to_owned());
    } else {
        eprintln!("Not in a project and project flag wasn't specified");
        exit(2);
    }
    serialize_goto_file(goto_file);
}

fn ls_cmd(all: bool) {
    let goto_file = deserialize_goto_file();
    let current_dir = env::current_dir().expect("Process has no working directory");

    if all {
        for project in goto_file.projects.iter() {
            pretty_print_project(project);
        }
    } else {
        if let Some(project) = goto_file.projects.iter().find(|project| current_dir.starts_with(&project.root)) {
            pretty_print_project(project);
        } else {
            eprintln!("Not currently in a project");
        }
    }
}

fn pretty_print_project(project: &Project) {
    println!("{}", project.root);
    for (alias, dir) in project.aliases.iter() {
        println!("\t{} = {}", alias, dir);
    }
}
