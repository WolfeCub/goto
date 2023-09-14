use std::io::ErrorKind;
use std::process::Command;
use std::sync::Arc;
use std::{borrow::Cow, fs};

use anyhow::Context;
use skim::prelude::{unbounded, SkimOptionsBuilder};
use skim::{ItemPreview, PreviewContext, SkimItem, SkimItemReceiver, SkimItemSender, Skim};

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
                    let files = fs::read_dir(&self.path)
                        .expect("Unable to read directory")
                        .map(|f| {
                            f.expect("Unable to read file")
                                .file_name()
                                .into_string()
                                .expect("Unable to read file name")
                        })
                        .collect::<Vec<_>>();
                    ItemPreview::Text(files.join("\n"))
                } else {
                    ItemPreview::Text(format!("Unable to preview directory contents: {}", e))
                }
            }
        }
    }
}

pub struct FuzzyMenu {
    tx_item: SkimItemSender,
    rx_item: SkimItemReceiver,
}

impl FuzzyMenu {
    pub fn new() -> FuzzyMenu {
        let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();

        Self { tx_item, rx_item }
    }

    pub fn add_option(&self, item: NicknamedDir) -> anyhow::Result<()> {
        self.tx_item.send(Arc::new(item)).with_context(|| "Unable to send item to fuzzy matcher")
    }

    pub fn run(self) -> anyhow::Result<NicknamedDir> {
        drop(self.tx_item); // so that skim could know when to stop waiting for more items.

        let options = SkimOptionsBuilder::default()
            .height(Some("50%"))
            .preview(Some(""))
            .build()
            .with_context(|| "Unable to build fuzzy matcher options")?;

        let selected_items = Skim::run_with(&options, Some(self.rx_item))
            .map(|out| out.selected_items)
            .unwrap_or_else(|| vec![]);

        assert!(selected_items.len() == 1);
        let selected_dir: &NicknamedDir = (*selected_items[0])
            .as_any()
            .downcast_ref::<NicknamedDir>()
            .with_context(|| "Unable to downcast selected fuzzy result to `NicknamedDir`")?;

        Ok((*selected_dir).clone())
    }
}
