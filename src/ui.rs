use std::{collections::BTreeMap, iter};

use chrono::NaiveDate;
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

use crate::{
    app::{Action, App, JournalMode, Pane, update},
    entry::Day,
    theme::{BackgroundArt, TITLE, Theme},
};

const CONT: &str = "         ";

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
        Some(Pane::Capture {
            text,
            character_index,
        }) => {
            background(frame);
            draw_capture_poopup(frame, text, character_index, &app.theme);
        }
        // TODO: This Archive and Journal will sahre a bunch of visual code for now.
        // Once I get this working, Ill clean it up, promisse to myself
        Some(Pane::Archive(_)) => todo!("archive view"),
        Some(Pane::Journal { selected, mode }) => {
            background(frame);
            draw_journal(frame, &app.days, mode, selected, &app.theme);
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
    height: Constraint,
    width: Constraint,
    frame: &mut Frame,
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
        Constraint::Length(7),
        Constraint::Percentage(65),
        frame,
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

fn draw_journal(
    frame: &mut Frame,
    days: &BTreeMap<NaiveDate, Day>,
    mode: &mut JournalMode,
    selected: &NaiveDate,
    theme: &Theme,
) {
    let main_block_inner_margin = draw_main_frame(
        Constraint::Percentage(90),
        Constraint::Percentage(90),
        frame,
        theme,
    );

    // Area for:
    //   1 - Header
    //   2 -Journal/Archive body
    //   3 - Footer
    let main_block_rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Length(1),
    ])
    .split(main_block_inner_margin);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            TITLE,
            Style::new().fg(theme.title).add_modifier(Modifier::BOLD),
        ))),
        main_block_rows[0],
    );

    let [workspace_sidepane, workspace_body] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(4)])
            .spacing(Spacing::Overlap(1))
            .areas(main_block_rows[1]);

    let sidepane_focused = matches!(mode, JournalMode::BrowsingSidepane);

    let pane_block = |block, fg| {
        Block::bordered()
            .borders(block)
            .border_type(BorderType::Thick)
            .border_style(Style::new().fg(fg))
            .merge_borders(MergeStrategy::Exact)
    };

    let workspace_sidepane_block = pane_block(
        Borders::TOP | Borders::BOTTOM | Borders::RIGHT,
        if sidepane_focused {
            theme.divider_focus
        } else {
            theme.divider
        },
    );

    let workspace_body_block = pane_block(
        Borders::TOP | Borders::BOTTOM | Borders::LEFT,
        if sidepane_focused {
            theme.divider
        } else {
            theme.divider_focus
        },
    );

    if sidepane_focused {
        frame.render_widget(&workspace_body_block, workspace_body);
        frame.render_widget(&workspace_sidepane_block, workspace_sidepane);
    } else {
        frame.render_widget(&workspace_sidepane_block, workspace_sidepane);
        frame.render_widget(&workspace_body_block, workspace_body);
    }

    let workspace_sidepane_block_inner = workspace_sidepane_block.inner(workspace_sidepane);

    let [workspace_sidepane_title, _, workspace_sidepane_content] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(workspace_sidepane_block_inner);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Journal",
            Style::new().fg(theme.text_dim).add_modifier(Modifier::BOLD),
        ))),
        workspace_sidepane_title,
    );

    let daily_records: Vec<(String, String)> = days
        .iter()
        .rev()
        .map(|(date, record)| (date.to_string(), record.records.len().to_string()))
        .collect();

    // Conditional formating for the slected jounral row. Changes color
    // based on selected row, and focused pane
    let selected_paragraph = |date_matched: bool, date: String, count: String| {
        let w = workspace_sidepane_content.width as usize;

        let (gutter, style) = match (date_matched, sidepane_focused) {
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

        let pad = w.saturating_sub(date.len() + count.len() + 3);

        Line::from(vec![
            gutter,
            Span::styled(date, style),
            Span::raw(" ".repeat(pad)),
            Span::styled(count, style),
            Span::raw(" "),
        ])
        .style(style)
    };

    let daily_paragraph = Paragraph::new(
        daily_records
            .into_iter()
            .map(|(date, count)| selected_paragraph(date == selected.to_string(), date, count))
            .collect::<Vec<_>>(),
    );

    frame.render_widget(daily_paragraph, workspace_sidepane_content);

    if let JournalMode::FocusedRecord { vertical_scroll } = mode
        && let Some(entry) = days.get(selected)
    {
        let workspace_body_inner = workspace_body.inner(Margin::new(2, 1));

        let [_, workspace_body_title, workspace_body_content] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(workspace_body_inner);

        // Padding for the inner body rendered
        let [inner] = Layout::horizontal([Constraint::Max(95)])
            .flex(Flex::Center)
            .areas(workspace_body_content.inner(Margin::new(0, 1)));
        let [text_area, bar_area] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(10)]).areas(inner);

        let wrap_widht = text_area.width.saturating_sub(CONT.chars().count() as u16);

        let entry_inner_rows: Vec<Line> = entry
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
                    textwrap::wrap_first_fit(&record_line, wrap_widht)
                        .into_iter()
                        .enumerate()
                        .map(|(i, mut line)| {
                            if i > 0 {
                                line.spans
                                    .insert(0, Span::styled(CONT, Style::new().fg(theme.text_dim)));
                            }
                            line
                        }),
                )
            })
            .collect();

        let viewport = text_area.height as usize;
        let max_scroll = entry_inner_rows.len().saturating_sub(viewport);
        *vertical_scroll = (*vertical_scroll).min(max_scroll);

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
            workspace_body_title,
        );

        // Rendering rows witing margin
        frame.render_widget(
            Paragraph::new(entry_inner_rows).scroll((*vertical_scroll as u16, 0)),
            text_area,
        );

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
                bar_area,
                &mut ScrollbarState::default()
                    .content_length(max_scroll + 1) // scroll positions: 0..=max_scroll
                    .viewport_content_length(viewport) // visible rows
                    .position(*vertical_scroll),
            );
        }
    }

    let footer_note = if sidepane_focused {
        journal_browsing_sidepane_instructions()
    } else {
        journal_focused_record_instructions()
    };

    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(footer_note, Style::new().fg(theme.text_dim))).right_aligned(),
        ),
        main_block_rows[2],
    );
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

fn journal_browsing_sidepane_instructions() -> &'static str {
    "j/k or ↑/↓ select day  ·  l or → open  ·  esc quit"
}

fn journal_focused_record_instructions() -> &'static str {
    "j/k or ↑/↓ scroll up or down  ·  h or ← back  ·  esc quit"
}

#[allow(dead_code)]
fn journal_capturing_instructions() -> &'static str {
    "h or ← back  ·  esc quit"
}

pub fn journal_browsing_sidepane_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Right | KeyCode::Char('l') => Some(Action::FocusRecord),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::PreviousDay),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::NextDay),
        KeyCode::Esc => Some(Action::Cancel),
        _ => None,
    }
}

pub fn journal_focused_record_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => Some(Action::FocusSidepane),
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Char('j') => Some(Action::ScrollbarNext),
        KeyCode::Char('k') => Some(Action::ScrollbarPrevious),
        _ => None,
    }
}

pub fn journal_capturing_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => Some(Action::FocusSidepane),
        KeyCode::Esc => Some(Action::Cancel),
        _ => None,
    }
}
