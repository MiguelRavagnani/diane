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

    // NOTE: This might bite me. If we are loading from disk
    // 365 notes for a year, for example, this sohuld be ok. But
    // lazily loading this should be smarter. Maybe move this to
    // BTreeMap later
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

    pub fn push_record(&mut self, text: String) {
        let at = chrono::Local::now().time();
        self.today_mut().records.push(Record { at, text });
    }

    pub fn commit_pair(&mut self) {
        // Mode::Browsing could be the default in the enum definition,
        // but i prefere to have this replace written explicitly.
        if let Mode::EditingPair { key, value, .. } =
            std::mem::replace(&mut self.mode, Mode::Browsing)
        {
            if key.trim().is_empty() {
                return;
            }
            if let Some(note) = self.selected_note_mut() {
                match note.frontmatter.iter_mut().find(|(k, _)| *k == key) {
                    Some(slot) => slot.1 = value,
                    None => note.frontmatter.push((key, value)),
                }
            }
        }
    }

    pub fn selected_note_mut(&mut self) -> Option<&mut Note> {
        self.notes.get_mut(self.note_cursor)
    }
}
