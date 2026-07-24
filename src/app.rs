use std::path::PathBuf;

use crate::config::Config;
use crate::entry::{Day, Note, Record};
use crate::stream::Vault;

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
    pub vault: Vault,

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
    pub fn new(config: &Config) -> App {
        Self {
            vault: Vault::new(config),
            days: Vec::new(),
            day_cursor: 0,
            notes: Vec::new(),
            note_cursor: 0,
            screen: Screen::Journal,
            mode: Mode::Browsing,
            should_quit: false,
        }
    }

    pub fn flush_journal(&mut self) -> std::io::Result<()> {
        let index = self.today_index();
        self.vault
            .append_journal_records(&mut self.days[index].records)
    }

    fn today_index(&mut self) -> usize {
        let today = chrono::Local::now().date_naive();
        match self.days.iter().position(|d| d.date == today) {
            Some(i) => i,
            None => {
                self.days.push(Day {
                    date: today,
                    records: Vec::new(),
                });
                self.days.len() - 1
            }
        }
    }

    // This keeps the days in check, making shure we get the days accesses and
    // modified as needed.
    pub fn today_mut(&mut self) -> &mut Day {
        let today_index = self.today_index();
        &mut self.days[today_index]
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
