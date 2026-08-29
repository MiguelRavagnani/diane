use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};

use chrono::{Local, NaiveDate};
use crossterm::event::KeyEvent;

use crate::config::Config;
use crate::entry::{Day, Note, Record};
use crate::stream::Vault;
use crate::theme::Theme;
use crate::ui::{
    archive_browsing_sidepane_action, archive_focused_record_action, capture_action,
    journal_browsing_sidepane_action, journal_focused_record_action,
};

pub enum Window {
    Library {
        selected_entry: NaiveDate,
        selected_note: usize,
        mode: Mode,
        focus: Focus,
        archive_scroll: usize,
        journal_scroll: usize,
        content_scroll: usize,
    },
    Capture {
        text: String,
        character_index: u16,
    },
}

pub enum Focus {
    Sidepane,
    Content,
}

pub enum Mode {
    Journal,
    Archive,
}

pub enum Field {
    Key,
    Value,
}

pub enum Action {
    Backspace,
    Cancel,
    CommitCapture,
    InsertChar(char),
    NextJournalEntry,
    NextArchiveNote,
    PreviousJournalEntry,
    PreviousArchiveNote,
    ScrollbarNext,
    ScrollbarPrevious,
    ToggleFocus,
    ToggleMode,
}

pub fn update(app: &mut App, action: Action) -> bool {
    match action {
        Action::ScrollbarNext => {
            if let Some(Window::Library {
                focus,
                content_scroll,
                journal_scroll: sidepane_scroll,
                ..
            }) = &mut app.pane
            {
                let scroll = match focus {
                    Focus::Content => content_scroll,
                    Focus::Sidepane => sidepane_scroll,
                };
                *scroll += 1;
            }
            false
        }
        Action::ScrollbarPrevious => {
            if let Some(Window::Library {
                focus,
                content_scroll,
                journal_scroll: sidepane_scroll,
                ..
            }) = &mut app.pane
            {
                let scroll = match focus {
                    Focus::Content => content_scroll,
                    Focus::Sidepane => sidepane_scroll,
                };
                *scroll = scroll.saturating_sub(1);
            }
            false
        }
        Action::ToggleMode => {
            if let Some(Window::Library { mode, .. }) = &mut app.pane {
                *mode = match mode {
                    Mode::Archive => Mode::Journal,
                    Mode::Journal => Mode::Archive,
                }
            }
            false
        }
        Action::ToggleFocus => {
            if let Some(Window::Library {
                focus,
                content_scroll,
                ..
            }) = &mut app.pane
            {
                *focus = match focus {
                    Focus::Content => {
                        *content_scroll = 0;
                        Focus::Sidepane
                    }
                    Focus::Sidepane => Focus::Content,
                }
            }
            false
        }
        Action::PreviousJournalEntry => {
            if let Some(Window::Library {
                selected_entry: selected,
                journal_scroll: scroll,
                ..
            }) = &mut app.pane
                && let Some((&date, _)) = app.days.range(..*selected).next_back()
            {
                *selected = date;
                *scroll += 1;
            }
            false
        }
        Action::NextJournalEntry => {
            if let Some(Window::Library {
                selected_entry: selected,
                journal_scroll: scroll,
                ..
            }) = &mut app.pane
                && let Some((&date, _)) = app.days.range((Excluded(*selected), Unbounded)).next()
            {
                *selected = date;
                *scroll = scroll.saturating_sub(1);
            }
            false
        }
        Action::PreviousArchiveNote => {
            if let Some(Window::Library {
                selected_note: selected,
                archive_scroll: scroll,
                ..
            }) = &mut app.pane
            {
                *selected = selected.saturating_sub(1);
                *scroll += 1;
            }
            false
        }
        Action::NextArchiveNote => {
            if let Some(Window::Library {
                selected_note: selected,
                archive_scroll: scroll,
                ..
            }) = &mut app.pane
            {
                *selected = (*selected + 1).min(app.notes.len().saturating_sub(1));
                *scroll = scroll.saturating_sub(1);
            }
            false
        }
        Action::InsertChar(c) => match &mut app.pane {
            Some(pane) => {
                if let Window::Capture {
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
                if let Window::Capture {
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
    pub pane: Option<Window>,
    pub should_quit: bool,

    pub theme: Theme,
}

impl App {
    pub fn new(config: &Config) -> App {
        Self {
            vault: Vault::new(config),
            days: BTreeMap::new(),
            notes: Vec::new(),
            note_cursor: 0,
            pane: Some(Window::Library {
                selected_entry: Local::now().date_naive(),
                selected_note: 0,
                mode: Mode::Journal,
                focus: Focus::Sidepane,
                archive_scroll: 0,
                journal_scroll: 0,
                content_scroll: 0,
            }),
            should_quit: false,
            theme: Theme::named(&config.theme),
        }
    }

    pub fn to_action(&self, key: KeyEvent) -> Option<Action> {
        self.pane.as_ref().and_then(|pane| match pane {
            Window::Capture { .. } => capture_action(key),
            Window::Library {
                mode: Mode::Journal,
                focus: Focus::Sidepane,
                ..
            } => journal_browsing_sidepane_action(key),
            Window::Library {
                mode: Mode::Journal,
                focus: Focus::Content,
                ..
            } => journal_focused_record_action(key),
            Window::Library {
                mode: Mode::Archive,
                focus: Focus::Sidepane,
                ..
            } => archive_browsing_sidepane_action(key),
            Window::Library {
                mode: Mode::Archive,
                focus: Focus::Content,
                ..
            } => archive_focused_record_action(key),
        })
    }

    pub fn retrieve_archive(&mut self) -> std::io::Result<()> {
        self.notes = self.vault.recover_archive_notes()?;
        if let Some(Window::Library {
            selected_note: selected,
            ..
        }) = &mut self.pane
        {
            *selected = self.notes.len().saturating_sub(1);
        }

        Ok(())
    }

    pub fn retrieve_journal(&mut self) -> std::io::Result<()> {
        self.days = self.vault.recover_journal_records()?;

        if let Some(Window::Library {
            selected_entry: selected,
            ..
        }) = &mut self.pane
            && let Some((&date, _)) = self.days.range(..=*selected).next_back()
        {
            *selected = date;
        }

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
        let Some(Window::Capture { text, .. }) = &self.pane else {
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
