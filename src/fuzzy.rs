use std::io::ErrorKind;
use std::marker::PhantomData;
use std::process::Command;
use std::sync::Arc;
use std::{borrow::Cow, fs};

use anyhow::Context;
use skim::prelude::{unbounded, SkimOptionsBuilder};
use skim::{ItemPreview, PreviewContext, Skim, SkimItem, SkimItemReceiver, SkimItemSender};

use crate::file::{deserialize_goto_file, Project};

#[derive(Clone, Debug)]
pub struct NicknamedDir {
    pub short: String,
    pub path: String,
}

impl NicknamedDir {
    pub fn new(nickname: &str, name: &str, path: &str) -> Self {
        Self {
            short: format!("{nickname}/{name}"),
            path: format!("{path}/{name}"),
        }
    }
}

impl SkimItem for NicknamedDir {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.short)
    }

    /* TODO: Make previews nicer (colored, tree) */
    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let result = Command::new("tree")
            .args(["-C", "-L", "2", &self.path])
            .output();

        match result {
            Ok(output) => {
                ItemPreview::AnsiText(String::from_utf8_lossy(&output.stdout).into_owned())
            }
            Err(e) => {
                if let ErrorKind::NotFound = e.kind() {
                    let files = list_files(&self.path);
                    ItemPreview::Text(files.join("\n"))
                } else {
                    ItemPreview::Text(format!("Unable to preview directory contents: {}", e))
                }
            }
        }
    }
}

fn list_files(path: &str) -> Vec<String> {
    fs::read_dir(path)
        .expect("Unable to read directory")
        .map(|f| {
            f.expect("Unable to read file")
                .file_name()
                .into_string()
                .expect("Unable to read file name")
        })
        .collect::<Vec<_>>()
}

pub struct FuzzyMenu<T> {
    tx_item: SkimItemSender,
    rx_item: SkimItemReceiver,
    _t: PhantomData<T>,
}

impl<T> FuzzyMenu<T>
where
    T: SkimItem + Clone,
{
    pub fn new() -> FuzzyMenu<T> {
        let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

        Self {
            tx_item,
            rx_item,
            _t: PhantomData,
        }
    }

    pub fn add_option(&self, item: T) -> anyhow::Result<()> {
        self.tx_item
            .send(Arc::new(item))
            .with_context(|| "Unable to send item to fuzzy matcher")
    }

    pub fn run(self) -> anyhow::Result<T> {
        drop(self.tx_item); // so that skim could know when to stop waiting for more items.

        let options = SkimOptionsBuilder::default()
            .height(Some("50%"))
            .preview(Some(""))
            .multi(false)
            .build()
            .with_context(|| "Unable to build fuzzy matcher options")?;

        let selected_items = Skim::run_with(&options, Some(self.rx_item))
            .map(|out| out.selected_items)
            .unwrap_or_else(|| vec![]);

        assert!(selected_items.len() == 1);
        let selected_dir: &T = (*selected_items[0])
            .as_any()
            .downcast_ref::<T>()
            .with_context(|| "Unable to downcast selected fuzzy result to `NicknamedDir`")?;

        Ok((*selected_dir).clone())
    }
}

impl SkimItem for Project {
    fn text(&self) -> Cow<str> {
        self.root.as_str().into()
    }
}

pub fn fuzzy_select_project() -> anyhow::Result<Project> {
    let wrapper = FuzzyMenu::<Project>::new();
    let config_file = deserialize_goto_file()?;

    for p in config_file.projects {
        wrapper.add_option(p)?;
    }

    wrapper.run()
}
