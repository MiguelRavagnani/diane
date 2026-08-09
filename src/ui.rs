use std::collections::BTreeMap;

use chrono::NaiveDate;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Flex, Layout, Margin, Position, Rect, Spacing},
    style::{Modifier, Style, Stylize},
    symbols::merge::MergeStrategy,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Fill, Paragraph},
};

use crate::{
    app::{Action, App, JournalMode, Pane, update},
    entry::Day,
    theme::{BackgroundArt, Theme},
};

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

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .spacing(Spacing::Overlap(1))
    .split(area);

    let horizontal = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .spacing(Spacing::Overlap(1))
    .split(vertical[1]);

    horizontal[1]
}

fn proportinal_centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    let horizontal = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1]);

    horizontal[1]
}

fn draw(frame: &mut Frame, app: &App) {
    match &app.pane {
        Some(Pane::Capture {
            text,
            character_index,
        }) => {
            background(frame);
            draw_capture_poopup(frame, text, character_index);
        }
        // TODO: This Archive and Journal will sahre a bunch of visual code for now.
        // Once I get this working, Ill clean it up, promisse to myself
        Some(Pane::Archive(_)) => todo!("archive view"),
        Some(Pane::Journal { selected, mode }) => {
            background(frame);
            draw_journal(frame, &app.days, mode, selected);
        }
        None => {}
    }
}

fn background(frame: &mut Frame) {
    let area = frame.area();
    frame.render_widget(
        BackgroundArt {
            amp: 3,
            slope: 2,
            spacing: 7,
            thickness: 4,
            ..Default::default()
        },
        area,
    );
}

fn draw_capture_poopup(frame: &mut Frame, text: &str, character_index: &u16) {
    let area = centered_rect(65, 7, frame.area());
    let diane_theme = Theme::default();

    // Im starting to get lost, so Ill leave some comments here about the
    // damn UI
    //
    // Clears the area behind the to-be-rendered background slpash
    frame.render_widget(Clear, area);

    // Render the splash
    frame.render_widget(Block::default().bg(diane_theme.bg), area);

    // Main block. Will hold the capture popup
    let [gutter, main_block] =
        Layout::horizontal([Constraint::Length(1), Constraint::Min(0)]).areas(area);

    let main_block_inner = main_block.inner(Margin::new(2, 1));

    frame.render_widget(Block::default().bg(diane_theme.bg), area);
    frame.render_widget(
        Fill::new("▌").style(Style::new().fg(diane_theme.border)),
        gutter,
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
            .border_style(Style::new().fg(diane_theme.divider)),
        main_block_inner_rows[1],
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "◮◮  DIANE:",
            Style::new()
                .fg(diane_theme.title)
                .add_modifier(Modifier::BOLD),
        ))),
        main_block_inner_rows[0],
    );
    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                "esc to cancel",
                Style::new().fg(diane_theme.text_dim),
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
                Style::new()
                    .fg(diane_theme.hint)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(text, Style::new().fg(diane_theme.text)),
        ])),
        input,
    );

    frame.set_cursor_position(Position::new(input.x + 3 + character_index, input.y));
}

fn draw_journal(
    frame: &mut Frame,
    days: &BTreeMap<NaiveDate, Day>,
    mode: &JournalMode,
    selected: &NaiveDate,
) {
    let area = proportinal_centered_rect(90, 90, frame.area());
    let diane_theme = Theme::default();

    frame.render_widget(Clear, area);

    let [gutter, main_block] =
        Layout::horizontal([Constraint::Length(1), Constraint::Min(0)]).areas(area);

    let main_block_inner_margin = main_block.inner(Margin::new(2, 1));

    frame.render_widget(Block::default().bg(diane_theme.bg), area);
    frame.render_widget(
        Fill::new("▌").style(Style::new().fg(diane_theme.border)),
        gutter,
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
            "◭◭  DIANE:",
            Style::new()
                .fg(diane_theme.title)
                .add_modifier(Modifier::BOLD),
        ))),
        main_block_rows[0],
    );
    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                "Header text placeholder",
                Style::new().fg(diane_theme.text_dim),
            ))
            .right_aligned(),
        ),
        main_block_rows[0],
    );

    let [workspace_sidepane, workspace_body] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(4)])
            .spacing(Spacing::Overlap(1))
            .areas(main_block_rows[1]);

    let sidepane_focused = matches!(mode, JournalMode::Browsing);

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
            diane_theme.divider_focus
        } else {
            diane_theme.divider
        },
    );

    let workspace_body_block = pane_block(
        Borders::TOP | Borders::BOTTOM | Borders::LEFT,
        if sidepane_focused {
            diane_theme.divider
        } else {
            diane_theme.divider_focus
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
            Style::new()
                .fg(diane_theme.text_dim)
                .add_modifier(Modifier::BOLD),
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
    let selected_paragraph =
        |date_matched: bool, date: String, count: String, sidepane_focused: bool| {
            let w = workspace_sidepane_content.width as usize;

            let (bg, fg) = if sidepane_focused {
                (diane_theme.selected_bg, diane_theme.selected_fg)
            } else {
                (diane_theme.selected_bg_dim, diane_theme.text_dim)
            };

            let gutter = if !sidepane_focused && date_matched {
                Span::styled("▌", Style::new().fg(diane_theme.border))
            } else {
                Span::raw(" ")
            };

            let style = if date == selected.to_string() {
                Style::new().bg(bg).fg(fg).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(diane_theme.text)
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
            .map(|(date, count)| {
                selected_paragraph(date == selected.to_string(), date, count, sidepane_focused)
            })
            .collect::<Vec<_>>(),
    );

    frame.render_widget(daily_paragraph, workspace_sidepane_content);

    if !sidepane_focused {
        let workspace_body_inner = workspace_body.inner(Margin::new(2, 1));

        let [workspace_body_title, _, workspace_body_content] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(workspace_body_inner);

        frame.render_widget(
            Paragraph::new(
                Line::from(Span::styled(
                    selected.to_string(),
                    Style::new()
                        .fg(diane_theme.text_dim)
                        .add_modifier(Modifier::BOLD),
                ))
                .centered(),
            ),
            workspace_body_title,
        );

        match days.get(selected) {
            Some(entry) => {
                let entry_inner_rows: Vec<Line> = entry
                    .records
                    .iter()
                    .flat_map(|record| {
                        vec![
                            Line::default(),
                            Line::from(vec![
                                Span::styled("❖ ", Style::new().fg(diane_theme.hint)),
                                Span::styled(
                                    record.at.format("%H:%M").to_string(),
                                    Style::new().fg(diane_theme.text_dim),
                                ),
                                Span::styled(": ", Style::new().fg(diane_theme.text_dim)),
                                Span::styled(&record.text, Style::new().fg(diane_theme.text)),
                            ]),
                        ]
                    })
                    .collect();
                frame.render_widget(
                    Paragraph::new(entry_inner_rows),
                    workspace_body_content.inner(Margin::new(10, 1)),
                );
            }
            None => todo!(),
        }

        // .iter()
        // .rev()
        // .map(|(date, record)| (date.to_string(), record.records.len().to_string()))
        // .collect();
    }

    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                "Footer text placeholder",
                Style::new().fg(diane_theme.text_dim),
            ))
            .right_aligned(),
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

pub fn journal_browsing_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Right => Some(Action::FocusBody),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::PreviousDay),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::NextDay),
        KeyCode::Esc => Some(Action::Cancel),
        _ => None,
    }
}

pub fn journal_capturing_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Left => Some(Action::FocusSidepane),
        KeyCode::Esc => Some(Action::Cancel),
        _ => None,
    }
}
