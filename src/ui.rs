use std::collections::BTreeMap;

use chrono::NaiveDate;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Flex, Layout, Margin, Position, Rect, Spacing},
    style::{Modifier, Style, Stylize},
    symbols::merge::MergeStrategy,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
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
            draw_capture_poopup(frame, text, character_index);
        }
        // TODO: This Archive and Journal will sahre a bunch of visual code for now.
        // Once I get this working, Ill clean it up, promisse to myself
        Some(Pane::Archive(_)) => todo!("archive view"),
        Some(Pane::Journal(JournalMode::Browsing)) => {
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
            draw_journal(frame, &app.days);
        }
        Some(Pane::Journal(JournalMode::Capturing { .. })) => todo!("capturing journal view"),
        None => {}
    }
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
    frame.render_widget(Block::default().bg(diane_theme.splash_bg), area);

    // Main block. Will hold the capture popup
    let main_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(diane_theme.border));

    let main_block_inner = main_block.inner(area).inner(Margin::new(2, 0));

    frame.render_widget(main_block, area);

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
            .border_style(Style::new().fg(diane_theme.spacer)),
        main_block_inner_rows[1],
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "DIANE:",
            Style::new()
                .fg(diane_theme.section_title)
                .add_modifier(Modifier::BOLD),
        ))),
        main_block_inner_rows[0],
    );
    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                "esc to cancel",
                Style::new().fg(diane_theme.info_text),
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
                " > ",
                Style::new()
                    .fg(diane_theme.hint_text)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(text, Style::new().fg(diane_theme.input_text)),
        ])),
        input,
    );

    frame.set_cursor_position(Position::new(input.x + 3 + character_index, input.y));
}

fn draw_journal(frame: &mut Frame, days: &BTreeMap<NaiveDate, Day>) {
    let area = proportinal_centered_rect(90, 90, frame.area());
    let diane_theme = Theme::default();

    frame.render_widget(Clear, area);
    // NOTE: The slpash might be too much. Maybe Ill remove it for the jorunal
    // archive window
    frame.render_widget(Block::default().bg(diane_theme.splash_bg), area);

    let main_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(diane_theme.border));
    let main_block_inner = main_block.inner(area);

    frame.render_widget(main_block, area);

    let main_block_inner_margin = main_block_inner.inner(Margin::new(2, 0));

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
            "DIANE:",
            Style::new()
                .fg(diane_theme.section_title)
                .add_modifier(Modifier::BOLD),
        ))),
        main_block_rows[0],
    );
    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                "Header text placeholder",
                Style::new().fg(diane_theme.info_text),
            ))
            .right_aligned(),
        ),
        main_block_rows[0],
    );

    let [workspace_sidepane, workspace_body] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(4)])
            .spacing(Spacing::Overlap(1))
            .areas(main_block_rows[1]);

    let [_, workspace_sidepane_title, workspace_sidepane_content] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(workspace_sidepane);

    frame.render_widget(
        Block::bordered()
            .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT)
            .border_style(Style::new().fg(diane_theme.spacer))
            .merge_borders(MergeStrategy::Exact),
        workspace_sidepane,
    );

    frame.render_widget(
        Block::bordered()
            .borders(Borders::TOP | Borders::BOTTOM | Borders::LEFT)
            .border_style(Style::new().fg(diane_theme.spacer))
            .merge_borders(MergeStrategy::Exact),
        workspace_body,
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Journal",
            Style::new().fg(diane_theme.section_title),
        ))),
        workspace_sidepane_title,
    );

    let daily_records: Vec<(String, String)> = days
        .iter()
        .rev()
        .map(|(date, record)| (date.to_string(), record.records.len().to_string()))
        .collect();

    let w = workspace_sidepane_content.width.saturating_sub(2) as usize;

    let daily_paragraph = Paragraph::new(
        daily_records
            .into_iter()
            .map(|(date, count)| {
                let pad = w.saturating_sub(date.len() + count.len());
                Line::from(vec![
                    Span::styled(date, Style::new().fg(diane_theme.info_text)),
                    Span::raw(" ".repeat(pad)),
                    Span::styled(count, Style::new().fg(diane_theme.info_text)),
                ])
            })
            .collect::<Vec<_>>(),
    );

    frame.render_widget(daily_paragraph, workspace_sidepane_content);

    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                "Footer text placeholder",
                Style::new().fg(diane_theme.info_text),
            ))
            .right_aligned(),
        ),
        main_block_rows[2],
    );
}

pub fn popup_capture_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char(c) => Some(Action::InsertChar(c)),
        KeyCode::Backspace => Some(Action::Backspace),
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Enter => Some(Action::CommitCapture),
        _ => None,
    }
}
