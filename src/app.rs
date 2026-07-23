use chrono::{NaiveDate, NaiveTime};
use std::path::PathBuf;

// TODO: Rethink Names here
pub enum Pane {
    PopUp,
    Journal,
    Archive,
}

// NOTE: Not sure if this should stay
pub enum CurrentlyEditing {
    ArchiveEntryBody,
    ArchiveEntryTitle,
}

pub struct Note {
    pub title: String,
    pub body: String,
    pub frontmatter: Vec<(String, String)>,
}

pub struct Record {
    pub at: NaiveTime,
    pub text: String,
}

pub struct Day {
    pub date: NaiveDate,
    pub records: Vec<Record>,
}

pub enum Screen {
    Journal,
    Archive,
}

pub enum Mode {
    Browsing,
    Capturing {
        text: String,
    },
    EditingPair {
        key: String,
        value: String,
        field: Field,
    },
}

pub enum Field {
    Key,
    Value,
}

pub struct App {
    pub vault: PathBuf,

    pub days: Vec<Day>,
    pub day_cursor: usize,

    pub notes: Vec<Note>,
    pub note_cursor: usize,

    pub screen: Screen,
    pub mode: Mode,
    pub should_quit: bool,
}

impl App {
    pub fn new(vault: PathBuf) -> App {
        Self {
            vault,
            days: Vec::new(),
            day_cursor: 0,
            notes: Vec::new(),
            note_cursor: 0,
            screen: Screen::Journal,
            mode: Mode::Browsing,
            should_quit: false,
        }
    }

    pub fn today_mut(&mut self) -> &mut Day {
        todo!()
    }

    pub fn selected_note_mut(&mut self) -> Option<&mut Note> {
        self.notes.get_mut(self.note_cursor)
    }
}
