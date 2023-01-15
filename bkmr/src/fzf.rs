use std::borrow::Cow;
use skim::prelude::*;
use std::io::Cursor;
use std::sync::Arc;
use skim::{AnsiString, DisplayContext, ItemPreview, PreviewContext, Skim, SkimItem, SkimItemReceiver, SkimItemSender};
use log::debug;
use stdext::function_name;
use termcolor::Ansi;
use tuikit::prelude::*;
use crate::models::Bookmark;

impl SkimItem for Bookmark {
    fn text(&self) -> Cow<str> {
        // let _text = format!("[{}] {}, {}", &self.id, &self.metadata, &self.URL);
        let _text = format!("[{}] {}, {}", self.id, self.metadata, self.URL);  // same??
        Cow::Owned(_text)
        // Cow::Borrowed(_text.as_str())
    }
    fn display<'a>(&'a self, context: DisplayContext<'a>) -> AnsiString<'a> {
        let start_idx_metadata = self.id.to_string().len() + 2;
        let end_idx_metadata = start_idx_metadata + self.metadata.len() + 1;
        let attr_metadata = Attr {
            fg: Color::GREEN,
            // bg: Color::Rgb(5, 10, 15),
            ..Attr::default()
        };
        let start_idx_url = end_idx_metadata + 1;
        let end_idx_url = start_idx_url + self.URL.len() + 1;
        let attr_url = Attr {
            fg: Color::BLUE,
            ..Attr::default()
        };
        AnsiString::new_str(
            context.text,
            vec![
                (attr_metadata, (start_idx_metadata as u32, end_idx_metadata as u32)),
                (attr_url, (start_idx_url as u32, end_idx_url as u32)),
            ])
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let _text = format!("[{}] {}, {}", &self.id, &self.metadata, &self.URL);
        ItemPreview::AnsiText(format!("\x1b[31mhello:\x1b[m\n{}", _text))
    }
}

fn fake_delete_item(item: &str) {
    println!("Deleting item `{}`...", item);
}

fn fake_create_item(item: &str) {
    println!("Creating a new item `{}`...", item);
}

pub fn fzf(bms: &Vec<Bookmark>) -> i32 {
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(true)
        // For full list of accepted keywords see `parse_event` in `src/event.rs`.
        .bind(vec!["ctrl-o:accept"])
        .build()
        .unwrap();

    // send bookmarks to skim
    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
    for bm in bms {
        tx_item.send(Arc::new(bm.clone())).unwrap();  // todo check clone
    }
    drop(tx_item); // so that skim could know when to stop waiting for more items.

    Skim::run_with(&options, Some(rx_item)).map(|out| match out.final_key {
        // Delete each selected item
        Key::Backspace => out.selected_items.iter().for_each(|i| fake_delete_item(&i.text())),
        // Create a new item based on the query
        Key::Enter => fake_create_item(out.query.as_ref()),
        Key::Ctrl('o') => {
            debug!("({}:{}) query: {:?} cmd: {:?}", function_name!(), line!(), out.query, out.cmd);

            out.selected_items.iter().for_each(|i| {
                println!("{}{}", i.output(), "\n");
            });
            let selected_bms = out.selected_items.iter().
                map(|selected_item| (**selected_item).as_any().downcast_ref::<Bookmark>().unwrap().to_owned())
                .collect::<Vec<Bookmark>>();
            debug!("({}:{}) selected_bms: {:?}", function_name!(), line!(), selected_bms);
        }
        _ => (),
    });

    1 as i32
}