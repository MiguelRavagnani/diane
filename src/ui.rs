use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Clear, Paragraph},
};

use crate::app::{App, Mode};

pub fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    // TODO: This ill only work or capturing now.
    app.mode = Mode::Capturing {
        text: String::new(),
    };
    loop {
        terminal.draw(|frame| draw(frame, app))?;
        if let Event::Key(key) = event::read()?
            && handle_key(app, key)
        {
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
    if let Mode::Capturing { text } = &app.mode {
        draw_capture_poopup(frame, text);
    }
}

fn draw_capture_poopup(frame: &mut Frame, text: &str) {
    let col1 = Color::Rgb(0xef, 0xa0, 0x10);
    let col2 = Color::Rgb(0xd8, 0x69, 0x0b);

    let area = centered_rect(60, 3, frame.area());

    let block = Block::bordered()
        .title(" diane: ")
        .border_style(Style::default().fg(col2))
        .title_style(Style::default().fg(col2));

    let paragraph = Paragraph::new(Text::from(text))
        .style(Style::default().fg(col1))
        .block(block);

    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
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
    let Mode::Capturing { text } = &mut app.mode else {
        return;
    };

    match key.code {
        KeyCode::Char(c) => text.push(c),
        KeyCode::Backspace => {
            text.pop();
        }
        KeyCode::Esc => {
            app.mode = Mode::Browsing;
        }
        KeyCode::Enter => {
            app.commit_capture();
        }
        _ => {}
    }
}
