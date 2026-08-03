use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Flex, Layout, Margin, Position, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::theme::Theme;
use crate::{
    app::{App, Mode},
    theme::BackgroundArt,
};

pub fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    // TODO: This ill only work or capturing now.
    app.mode = Mode::Capturing {
        text: String::new(),
        character_index: 0,
    };
    loop {
        terminal.draw(|frame| draw(frame, app))?;
        if let Event::Key(key) = event::read()? {
            handle_key(app, key);
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
    if let Mode::Capturing {
        text,
        character_index,
    } = &app.mode
    {
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
}

fn draw_capture_poopup(frame: &mut Frame, text: &str, character_index: &u16) {
    let area = centered_rect(65, 7, frame.area());
    let diane_theme = Theme::default();

    frame.render_widget(Clear, area);
    frame.render_widget(Block::default().bg(diane_theme.splash_bg), area);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(diane_theme.border));
    let inner = block.inner(area);

    frame.render_widget(block, area);

    let inner_margin = inner.inner(Margin::new(2, 0));
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(inner_margin);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "DIANE:",
            Style::new()
                .fg(diane_theme.section_title)
                .add_modifier(Modifier::BOLD),
        ))),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(
            Line::from(Span::styled(
                "esc to cancel",
                Style::new().fg(diane_theme.info_text),
            ))
            .right_aligned(),
        ),
        rows[0],
    );
    frame.render_widget(
        Block::new()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Thick)
            .border_style(Style::new().fg(diane_theme.spacer)),
        Rect {
            x: inner.x,
            width: inner.width,
            height: 1,
            ..rows[1]
        },
    );

    let [input] = Layout::vertical([Constraint::Length(1)])
        .flex(Flex::Center)
        .areas(rows[2]);
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

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if key.kind != KeyEventKind::Press {
        return false;
    }

    match &mut app.mode {
        Mode::Capturing { .. } => {
            handle_capturing(app, key);
            false
        }
        _ => todo!(),
    }
}

fn handle_capturing(app: &mut App, key: KeyEvent) {
    let Mode::Capturing {
        text,
        character_index,
    } = &mut app.mode
    else {
        return;
    };

    match key.code {
        KeyCode::Char(c) => {
            text.push(c);
            *character_index += 1;
        }
        KeyCode::Backspace => {
            text.pop();
            *character_index = character_index.saturating_sub(1);
        }
        KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Enter => {
            app.commit_capture();
            app.flush_journal();
            app.should_quit = true;
        }
        _ => {}
    }
}
