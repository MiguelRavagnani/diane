use chrono::Local;
use crossterm::event::KeyEvent;

use crate::config::Config;
use crate::entry::{Day, Note, Record};
use crate::stream::Vault;
use crate::ui::popup_capture_action;

// NOTE: Not sure if this should stay
// pub enum CurrentlyEditing {
//     ArchiveEntryBody,
//     ArchiveEntryTitle,
// }

pub enum Pane {
    Archive(ArchiveMode),
    Journal(JournalMode),
    Capture { text: String, character_index: u16 },
}

pub enum JournalMode {
    Browsing,
    Capturing { text: String, character_index: u16 },
}

pub enum ArchiveMode {
    Browsing,
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

pub enum Action {
    InsertChar(char),
    Backspace,
    Cancel,
    CommitCapture,
}

pub fn update(app: &mut App, action: Action) -> bool {
    match action {
        Action::InsertChar(c) => {
            if let Pane::Capture {
                text,
                character_index,
            } = &mut app.pane
            {
                text.push(c);
                *character_index += 1;
            }
            false
        }
        Action::Backspace => {
            if let Pane::Capture {
                text,
                character_index,
            } = &mut app.pane
            {
                text.pop();
                *character_index = character_index.saturating_sub(1);
            }
            false
        }
        Action::Cancel => {
            app.should_quit = true;
            false
        }
        Action::CommitCapture => {
            app.commit_capture();
            app.should_quit = true;
            true
        }
    }
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

    pub pane: Pane,
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
            pane: Pane::Journal(JournalMode::Browsing),
            should_quit: false,
        }
    }

    pub fn to_action(&self, key: KeyEvent) -> Option<Action> {
        match &self.pane {
            Pane::Capture { .. } => popup_capture_action(key),
            _ => todo!(),
        }
    }
    pub fn flush_journal(&mut self) -> std::io::Result<()> {
        let index = self.today_index();
        self.vault
            .append_journal_records(&mut self.days[index].records)
    }

    fn today_index(&mut self) -> usize {
        let today = Local::now().date_naive();
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

    pub fn selected_note_mut(&mut self) -> Option<&mut Note> {
        self.notes.get_mut(self.note_cursor)
    }

    pub fn push_record(&mut self, text: String) {
        let at = chrono::Local::now().time();
        self.today_mut().records.push(Record { at, text });
    }

    pub fn save_current_note(&mut self) -> std::io::Result<()> {
        let Some(note) = self.notes.get_mut(self.note_cursor) else {
            return Ok(());
        };
        note.updated = Local::now().date_naive();
        self.vault.save_archive_note(note)
    }

    #[allow(dead_code)]
    fn capture_oneshot(&mut self, text: &str) -> std::io::Result<()> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(());
        }

        self.today_mut().records.push(Record {
            at: chrono::Local::now().time(),
            text: text.to_string(),
        });
        self.flush_journal()
    }

    pub fn commit_capture(&mut self) {
        let Pane::Capture { text, .. } = &self.pane else {
            return;
        };

        let text = text.trim().to_string();
        if text.is_empty() {
            return;
        }

        let record = Record {
            at: Local::now().time(),
            text,
        };

        self.today_mut().records.push(record);
    }
}
