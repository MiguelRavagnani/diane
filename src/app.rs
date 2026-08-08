use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};

use chrono::{Local, NaiveDate};
use crossterm::event::KeyEvent;

use crate::config::Config;
use crate::entry::{Day, Note, Record};
use crate::stream::Vault;
use crate::ui::{capture_action, journal_browsing_action, journal_capturing_action};

// NOTE: Not sure if this should stay
// pub enum CurrentlyEditing {
//     ArchiveEntryBody,
//     ArchiveEntryTitle,
// }

pub enum Pane {
    Archive(ArchiveMode),
    Journal {
        selected: NaiveDate,
        mode: JournalMode,
    },
    Capture {
        text: String,
        character_index: u16,
    },
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
    NextDay,
    FocusSidepane,
    FocusBody,
    PreviousDay,
    Backspace,
    Cancel,
    CommitCapture,
}

pub fn update(app: &mut App, action: Action) -> bool {
    match action {
        Action::FocusBody => {
            if let Some(Pane::Journal { selected: _, mode }) = &mut app.pane {
                *mode = JournalMode::Capturing {
                    text: String::new(),
                    character_index: 0,
                }
            }
            false
        }
        Action::FocusSidepane => {
            if let Some(Pane::Journal { selected: _, mode }) = &mut app.pane {
                *mode = JournalMode::Browsing
            }
            false
        }
        Action::PreviousDay => {
            if let Some(Pane::Journal { selected, mode: _ }) = &mut app.pane
                && let Some((&date, _)) = app.days.range(..*selected).next_back()
            {
                *selected = date;
            }
            false
        }
        Action::NextDay => {
            if let Some(Pane::Journal { selected, mode: _ }) = &mut app.pane
                && let Some((&date, _)) = app.days.range((Excluded(*selected), Unbounded)).next()
            {
                *selected = date;
            }
            false
        }
        Action::InsertChar(c) => match &mut app.pane {
            Some(pane) => {
                if let Pane::Capture {
                    text,
                    character_index,
                } = pane
                {
                    text.push(c);
                    *character_index += 1;
                }
                false
            }
            None => false,
        },
        Action::Backspace => match &mut app.pane {
            Some(pane) => {
                if let Pane::Capture {
                    text,
                    character_index,
                } = pane
                {
                    text.pop();
                    *character_index = character_index.saturating_sub(1);
                }
                false
            }
            None => false,
        },
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

    pub days: BTreeMap<NaiveDate, Day>,

    pub notes: Vec<Note>,
    pub note_cursor: usize,

    // NOTE: So I decided to make this Option for now. Currently,
    // this does nothing more than making me rewrite a bunch of
    // stuff for case matching, BUT I do think this will be usefull
    // soon. Maybe detach the CLI's text input matching for good...
    pub pane: Option<Pane>,
    pub should_quit: bool,
}

impl App {
    pub fn new(config: &Config) -> App {
        Self {
            vault: Vault::new(config),
            days: BTreeMap::new(),
            notes: Vec::new(),
            note_cursor: 0,
            pane: Some(Pane::Journal {
                selected: Local::now().date_naive(),
                mode: JournalMode::Browsing,
            }),
            should_quit: false,
        }
    }

    pub fn to_action(&self, key: KeyEvent) -> Option<Action> {
        self.pane.as_ref().and_then(|pane| match pane {
            Pane::Capture { .. } => capture_action(key),
            Pane::Archive(_) => todo!("Archive to_action"),
            Pane::Journal {
                selected: _,
                mode: JournalMode::Capturing { .. },
            } => journal_capturing_action(key),
            Pane::Journal {
                selected: _,
                mode: JournalMode::Browsing,
            } => journal_browsing_action(key),
        })
    }

    pub fn retrieve_journal(&mut self) -> std::io::Result<()> {
        self.days = self.vault.recover_journal_records()?;
        self.days.entry(Local::now().date_naive()).or_default();
        Ok(())
    }

    pub fn flush_journal(&mut self) -> std::io::Result<()> {
        let today = Local::now().date_naive();
        let day = self.days.entry(today).or_default();
        self.vault.append_journal_records(today, &mut day.records)
    }

    // This keeps the days in check, making shure we get the days accesses and
    // modified as needed.
    pub fn today_mut(&mut self) -> &mut Day {
        self.days.entry(Local::now().date_naive()).or_default()
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

    pub fn commit_capture(&mut self) {
        let Some(Pane::Capture { text, .. }) = &self.pane else {
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
