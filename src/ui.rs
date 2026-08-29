use std::{collections::BTreeMap, iter};

use chrono::NaiveDate;
use crossterm::event::KeyModifiers;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Flex, Layout, Margin, Position, Rect, Spacing},
    style::{Modifier, Style, Stylize},
    symbols::{merge::MergeStrategy, scrollbar::Set},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Fill, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
};
use ratatui_textwrap::algorithms::textwrap;
use tui_markdown::{Options, from_str_with_options};

use crate::{
    app::{Action, App, Focus, Mode, Window, update},
    entry::{Day, Note},
    theme::{BackgroundArt, TITLE, Theme},
};

const RECORD_INDENT: &str = "         ";

pub fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    while app.pane.is_some() {
        terminal.draw(|frame| draw(frame, app))?;
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && let Some(action) = app.to_action(key)
        {
            let needs_flush = update(app, action);
            if needs_flush {
                app.flush_journal()?;
            }
        }
        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn draw(frame: &mut Frame, app: &mut App) {
    match &mut app.pane {
        Some(Window::Capture {
            text,
            character_index,
        }) => {
            background(frame);
            draw_capture_poopup(frame, text, character_index, &app.theme);
        }
        Some(Window::Library {
            selected_entry,
            selected_note,
            mode,
            focus,
            archive_scroll,
            journal_scroll,
            content_scroll,
        }) => {
            background(frame);
            draw_library(
                frame,
                &app.days,
                &app.notes,
                mode,
                focus,
                archive_scroll,
                journal_scroll,
                content_scroll,
                selected_entry,
                selected_note,
                &app.theme,
            );
        }
        None => {}
    }
}

fn background(frame: &mut Frame) {
    frame.render_widget(
        BackgroundArt {
            amp: 3,
            slope: 2,
            spacing: 7,
            thickness: 4,
            ..Default::default()
        },
        frame.area(),
    );
}

fn draw_main_frame(
    frame: &mut Frame,
    width: Constraint,
    height: Constraint,
    theme: &Theme,
) -> Rect {
    let [area] = Layout::vertical([height])
        .flex(Flex::Center)
        .areas(frame.area());
    let [area] = Layout::horizontal([width]).flex(Flex::Center).areas(area);

    let [gutter, main_block] =
        Layout::horizontal([Constraint::Length(1), Constraint::Min(0)]).areas(area);

    let main_block_inner = main_block.inner(Margin::new(2, 1));

    frame.render_widget(Clear, area);
    frame.render_widget(Block::default().bg(theme.bg), area);
    frame.render_widget(Fill::new("▌").style(Style::new().fg(theme.border)), gutter);

    main_block_inner
}

fn draw_capture_poopup(frame: &mut Frame, text: &str, character_index: &u16, theme: &Theme) {
    let main_block_inner = draw_main_frame(
        frame,
        Constraint::Percentage(65),
        Constraint::Length(7),
        theme,
    );

    // here, the division is:
    //  1 - Header
    //  2 - Spacer
    //  3 - Input area body
    let main_block_inner_rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(main_block_inner);

    // Spacer hack
    frame.render_widget(
        Block::new()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Thick)
            .border_style(Style::new().fg(theme.divider)),
        main_block_inner_rows[1],
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            TITLE,
            Style::new().fg(theme.title).add_modifier(Modifier::BOLD),
        ))),
        main_block_inner_rows[0],
    );
    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                "esc to cancel",
                Style::new().fg(theme.text_dim),
            ))
            .right_aligned(),
        ),
        main_block_inner_rows[0],
    );

    let [input] = Layout::vertical([Constraint::Length(1)])
        .flex(Flex::Center)
        .areas(main_block_inner_rows[2]);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " ❯ ",
                Style::new().fg(theme.hint).add_modifier(Modifier::BOLD),
            ),
            Span::styled(text, Style::new().fg(theme.text)),
        ])),
        input,
    );

    frame.set_cursor_position(Position::new(input.x + 3 + character_index, input.y));
}

fn record_lines(entry: &Day, width: u16, theme: &Theme) -> Vec<Line<'static>> {
    entry
        .records
        .iter()
        .flat_map(|record| {
            let record_line = Line::from(vec![
                Span::styled("❖ ", Style::new().fg(theme.hint)),
                Span::styled(
                    record.at.format("%H:%M").to_string(),
                    Style::new().fg(theme.text_dim),
                ),
                Span::styled(": ", Style::new().fg(theme.text_dim)),
                Span::styled(&record.text, Style::new().fg(theme.text)),
            ]);

            iter::once(Line::default()).chain(
                textwrap::wrap_first_fit(&record_line, width)
                    .into_iter()
                    .enumerate()
                    .map(|(i, mut line)| {
                        if i > 0 {
                            line.spans.insert(
                                0,
                                Span::styled(RECORD_INDENT, Style::new().fg(theme.text_dim)),
                            );
                        }
                        line
                    }),
            )
        })
        .collect()
}

fn draw_note(frame: &mut Frame, area: Rect, note: &Note, scroll: &mut usize, theme: &Theme) {
    let [inner] = Layout::horizontal([Constraint::Min(90)])
        .flex(Flex::Center)
        .areas(area.inner(Margin::new(6, 1)));

    let [text_area, bar_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(10)]).areas(inner);

    let markdown_theme = Options::new(*theme);

    let note_text = &note.to_text();
    let rendered = from_str_with_options(note_text, &markdown_theme);

    let note_rows: Vec<Line> = rendered
        .lines
        .iter()
        .flat_map(|line| textwrap::wrap_first_fit(line, text_area.width))
        .collect();

    let scroll_constraint = note_rows.len();
    let note_paragraph = Paragraph::new(note_rows).style(Style::default().fg(theme.text));
    let viewport = text_area.height as usize;

    render_content_scrollbar(frame, scroll, scroll_constraint, viewport, bar_area, theme);

    frame.render_widget(note_paragraph.scroll((*scroll as u16, 0)), text_area);
}

fn draw_records(
    frame: &mut Frame,
    area: Rect,
    selected: &NaiveDate,
    entry: &Day,
    scroll: &mut usize,
    theme: &Theme,
) {
    let [title, content] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

    let [inner] = Layout::horizontal([Constraint::Min(90)])
        .flex(Flex::Center)
        .areas(content.inner(Margin::new(6, 1)));
    let [text_area, bar_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(10)]).areas(inner);

    let wrap_widht = text_area.width.saturating_sub(RECORD_INDENT.len() as u16);

    let entry_inner_rows = record_lines(entry, wrap_widht, theme);
    let viewport = text_area.height as usize;

    // Note content title. The date of the journal entry, and how many records It has
    frame.render_widget(
        Paragraph::new(
            Line::from(vec![
                Span::styled(
                    selected.to_string(),
                    Style::new().fg(theme.title).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  ·  {} entries", entry.records.len()),
                    Style::new().fg(theme.text_dim).add_modifier(Modifier::BOLD),
                ),
            ])
            .centered(),
        ),
        title,
    );

    render_content_scrollbar(
        frame,
        scroll,
        entry_inner_rows.len(),
        viewport,
        bar_area,
        theme,
    );

    // Rendering rows witing margin
    frame.render_widget(
        Paragraph::new(entry_inner_rows).scroll((*scroll as u16, 0)),
        text_area,
    );
}

fn render_sidepane_scrollbar(
    frame: &mut Frame,
    scroll: &mut usize,
    scroll_constraint: usize,
    viewport: usize,
    area: Rect,
    theme: &Theme,
) {
    let max_scroll = scroll_constraint.saturating_sub(viewport);
    *scroll = (*scroll).min(max_scroll);

    let [top, _, bottom] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    if *scroll > 0 {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("...", Style::new().fg(theme.hint)))).centered(),
            top,
        );
    }

    if *scroll < max_scroll {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("...", Style::new().fg(theme.hint)))).centered(),
            bottom,
        );
    }
}

fn render_content_scrollbar(
    frame: &mut Frame,
    scroll: &mut usize,
    scroll_constraint: usize,
    viewport: usize,
    area: Rect,
    theme: &Theme,
) {
    let max_scroll = scroll_constraint.saturating_sub(viewport);
    *scroll = (*scroll).min(max_scroll);

    if max_scroll > 0 {
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(Set {
                    track: " ",
                    thumb: ".",
                    begin: "▲",
                    end: "▼",
                })
                .style(Style::new().fg(theme.hint)),
            area,
            &mut ScrollbarState::default()
                .content_length(max_scroll + 1) // scroll positions: 0..=max_scroll
                .viewport_content_length(viewport) // visible rows
                .position(*scroll),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_library(
    frame: &mut Frame,
    days: &BTreeMap<NaiveDate, Day>,
    notes: &[Note],
    mode: &mut Mode,
    focus: &Focus,
    archive_scroll: &mut usize,
    journal_scroll: &mut usize,
    content_scroll: &mut usize,
    selected_entry: &NaiveDate,
    selected_note: &mut usize,
    theme: &Theme,
) {
    let main_area = draw_main_frame(
        frame,
        Constraint::Percentage(90),
        Constraint::Percentage(90),
        theme,
    );

    let [header_area, library_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Length(1),
    ])
    .areas(main_area);

    let [sidepane_area, content_area] =
        Layout::horizontal([Constraint::Max(25), Constraint::Fill(1)])
            .spacing(Spacing::Overlap(1))
            .areas(library_area);

    let [journal_sidepane_area, archive_sidepane_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)])
            .spacing(Spacing::Overlap(1))
            .areas(sidepane_area);

    let journal_sidepane_focused =
        matches!(mode, Mode::Journal) && matches!(focus, Focus::Sidepane);
    let archive_sidepane_focused =
        matches!(mode, Mode::Archive) && matches!(focus, Focus::Sidepane);
    let content_focused = matches!(focus, Focus::Content);

    let divider = |focused| {
        if focused {
            theme.divider_focus
        } else {
            theme.divider
        }
    };

    let journal_divider = divider(journal_sidepane_focused);
    let archive_divider = divider(archive_sidepane_focused);
    let content_divider = divider(content_focused);

    // I know the compiler will just turn this non-capturing closure into
    // a fn, but i dont think this is big or specialized enought to
    // be Its own function rn
    let pane_block = |block, fg| {
        Block::bordered()
            .borders(block)
            .border_type(BorderType::Thick)
            .border_style(Style::new().fg(fg))
            .merge_borders(MergeStrategy::Exact)
    };

    let journal_sidepane_panel_block = pane_block(
        Borders::TOP | Borders::BOTTOM | Borders::RIGHT,
        journal_divider,
    );

    let archive_sidepane_panel_block = pane_block(
        Borders::TOP | Borders::BOTTOM | Borders::RIGHT,
        archive_divider,
    );

    let content_panel_block = pane_block(
        Borders::TOP | Borders::BOTTOM | Borders::LEFT,
        content_divider,
    );

    // Encoding the data instead of branching. This ensures correct
    // rendering order, by sorting the FOCUSED pannel
    let mut panes = [
        (
            &journal_sidepane_panel_block,
            journal_sidepane_area,
            matches!(focus, Focus::Sidepane) && matches!(mode, Mode::Journal),
        ),
        (
            &archive_sidepane_panel_block,
            archive_sidepane_area,
            matches!(focus, Focus::Sidepane) && matches!(mode, Mode::Archive),
        ),
        (
            &content_panel_block,
            content_area,
            matches!(focus, Focus::Content),
        ),
    ];

    panes.sort_by_key(|(_, _, focused)| *focused);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            TITLE,
            Style::new().fg(theme.title).add_modifier(Modifier::BOLD),
        ))),
        header_area,
    );

    for (block, area, _) in panes {
        frame.render_widget(block, area);
    }

    draw_day_list(
        frame,
        journal_sidepane_panel_block.inner(journal_sidepane_area),
        selected_entry,
        days,
        journal_sidepane_focused,
        journal_scroll,
        theme,
    );

    draw_note_list(
        frame,
        archive_sidepane_panel_block.inner(archive_sidepane_area),
        notes,
        selected_note,
        archive_sidepane_focused,
        archive_scroll,
        theme,
    );

    if content_focused {
        match mode {
            Mode::Journal => {
                if let Some(entry) = days.get(selected_entry) {
                    draw_records(
                        frame,
                        content_area.inner(Margin::new(0, 1)),
                        selected_entry,
                        entry,
                        content_scroll,
                        theme,
                    );
                }
            }
            Mode::Archive => {
                draw_note(
                    frame,
                    content_area.inner(Margin::new(0, 1)),
                    &notes[*selected_note],
                    content_scroll,
                    theme,
                );
            }
        }
    }

    let footer_note = if journal_sidepane_focused {
        journal_sidepane_instructions()
    } else if archive_sidepane_focused {
        archive_sidepane_instructions()
    } else {
        content_instructions()
    };

    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(footer_note, Style::new().fg(theme.text_dim))).right_aligned(),
        ),
        footer_area,
    );
}

fn draw_day_list(
    frame: &mut Frame,
    area: Rect,
    selected: &NaiveDate,
    days: &BTreeMap<NaiveDate, Day>,
    focused: bool,
    scroll: &mut usize,
    theme: &Theme,
) {
    let [day_list_title_area, _, day_list_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);

    // Conditional formating for the slected jounral row. Changes color
    // based on selected row, and focused pane
    let day_list = Paragraph::new(
        days.iter()
            .rev()
            .map(|(date, day)| {
                let is_selected = date == selected;
                let date = date.to_string();
                let count = day.records.len().to_string();

                let (gutter, style_day, style_count) = match (is_selected, focused) {
                    (false, _) => (
                        Span::raw(" "),
                        Style::new().fg(theme.text),
                        Style::new().fg(theme.text_dim),
                    ),
                    (true, true) => (
                        Span::raw(" "),
                        Style::new()
                            .bg(theme.selected_bg)
                            .fg(theme.selected_fg)
                            .add_modifier(Modifier::BOLD),
                        Style::new().fg(theme.selected_fg),
                    ),
                    (true, false) => (
                        Span::styled("▌", Style::new().fg(theme.border)),
                        Style::new()
                            .bg(theme.selected_bg_dim)
                            .fg(theme.text_dim)
                            .add_modifier(Modifier::BOLD),
                        Style::new().fg(theme.text_dim),
                    ),
                };

                let pad =
                    (day_list_area.width as usize).saturating_sub(date.len() + count.len() + 3);

                Line::from(vec![
                    gutter,
                    Span::raw(date),
                    Span::raw(" ".repeat(pad)),
                    Span::styled(count, style_count),
                    Span::raw(" "),
                ])
                .style(style_day)
            })
            .collect::<Vec<_>>(),
    );

    let scroll_constraint = days.len();

    let [day_list_area, scrollbar_area] = if scroll_constraint > day_list_area.height as usize {
        [
            day_list_area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            day_list_area,
        ]
    } else {
        [day_list_area, Rect::ZERO]
    };

    let viewport = day_list_area.height as usize;

    render_sidepane_scrollbar(
        frame,
        scroll,
        scroll_constraint,
        viewport,
        scrollbar_area,
        theme,
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Journal",
            Style::new().fg(theme.text_dim).add_modifier(Modifier::BOLD),
        ))),
        day_list_title_area,
    );

    frame.render_widget(day_list.scroll((*scroll as u16, 0)), day_list_area);
}

fn draw_note_list(
    frame: &mut Frame,
    area: Rect,
    notes: &[Note],
    selected: &usize,
    focused: bool,
    scroll: &mut usize,
    theme: &Theme,
) {
    let [notes_title_area, _, notes_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);

    let note_list = Paragraph::new(
        notes
            .iter()
            .enumerate()
            .rev()
            .map(|(index, note)| {
                let is_selected = index == *selected;
                let title = note.title.clone();
                let (gutter, style_note) = match (is_selected, focused) {
                    (false, _) => (Span::raw(" "), Style::new().fg(theme.text)),
                    (true, true) => (
                        Span::raw(" "),
                        Style::new()
                            .bg(theme.selected_bg)
                            .fg(theme.selected_fg)
                            .add_modifier(Modifier::BOLD),
                    ),
                    (true, false) => (
                        Span::styled("▌", Style::new().fg(theme.border)),
                        Style::new()
                            .bg(theme.selected_bg_dim)
                            .fg(theme.text_dim)
                            .add_modifier(Modifier::BOLD),
                    ),
                };

                let title_render = match title
                    .char_indices()
                    .nth(notes_area.width.saturating_sub(2).into())
                {
                    Some((idx, _)) => format!("{}...", &title[..idx.saturating_sub(3)]),
                    None => title,
                };

                let pad = (notes_area.width as usize).saturating_sub(title_render.len() + 2);

                Line::from(vec![
                    gutter,
                    Span::raw(title_render),
                    Span::raw(" ".repeat(pad)),
                ])
                .style(style_note)
            })
            .collect::<Vec<_>>(),
    );

    let scroll_constraint = notes.len();

    let [notes_area, scrollbar_area] = if scroll_constraint > notes_area.height as usize {
        [
            notes_area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            notes_area,
        ]
    } else {
        [notes_area, Rect::ZERO]
    };

    let viewport = notes_area.height as usize;

    render_sidepane_scrollbar(
        frame,
        scroll,
        scroll_constraint,
        viewport,
        scrollbar_area,
        theme,
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Archive",
            Style::new().fg(theme.text_dim).add_modifier(Modifier::BOLD),
        ))),
        notes_title_area,
    );
    frame.render_widget(note_list.scroll((*scroll as u16, 0)), notes_area);
}

pub fn capture_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char(c) => Some(Action::InsertChar(c)),
        KeyCode::Backspace => Some(Action::Backspace),
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Enter => Some(Action::CommitCapture),
        _ => None,
    }
}

fn journal_sidepane_instructions() -> &'static str {
    "j/k or ↑/↓ select day  ·  l or → open  ·  SHIFT + j archive  ·  esc quit"
}

fn content_instructions() -> &'static str {
    "j/k or ↑/↓ scroll up or down  ·  h or ← back  ·  esc quit"
}

fn archive_sidepane_instructions() -> &'static str {
    "j/k or ↑/↓ select note  ·  l or → open  ·  SHIFT + k journal  ·  esc quit"
}

pub fn journal_browsing_sidepane_action(key: KeyEvent) -> Option<Action> {
    match (key.code, key.modifiers) {
        (KeyCode::Right | KeyCode::Char('l'), _) => Some(Action::ToggleFocus),
        (KeyCode::Down | KeyCode::Char('j'), KeyModifiers::NONE) => {
            Some(Action::PreviousJournalEntry)
        }
        (KeyCode::Up | KeyCode::Char('k'), _) => Some(Action::NextJournalEntry),
        (KeyCode::Char('J'), KeyModifiers::SHIFT | KeyModifiers::NONE) => Some(Action::ToggleMode),
        (KeyCode::Esc, _) => Some(Action::Cancel),
        _ => None,
    }
}

pub fn archive_browsing_sidepane_action(key: KeyEvent) -> Option<Action> {
    match (key.code, key.modifiers) {
        (KeyCode::Right | KeyCode::Char('l'), _) => Some(Action::ToggleFocus),
        (KeyCode::Down | KeyCode::Char('j'), KeyModifiers::NONE) => {
            Some(Action::PreviousArchiveNote)
        }
        (KeyCode::Up | KeyCode::Char('k'), _) => Some(Action::NextArchiveNote),
        (KeyCode::Char('K'), KeyModifiers::SHIFT | KeyModifiers::NONE) => Some(Action::ToggleMode),
        (KeyCode::Esc, _) => Some(Action::Cancel),
        _ => None,
    }
}

pub fn journal_focused_record_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => Some(Action::ToggleFocus),
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::ScrollbarNext),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::ScrollbarPrevious),
        _ => None,
    }
}

pub fn archive_focused_record_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => Some(Action::ToggleFocus),
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::ScrollbarNext),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::ScrollbarPrevious),
        _ => None,
    }
}
