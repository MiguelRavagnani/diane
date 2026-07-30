use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position, Rect},
    style::Style,
    text::Text,
    widgets::{Block, Clear, Paragraph},
};

use crate::app::{App, Mode};
use crate::theme;

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
        draw_capture_poopup(frame, text, character_index);
    }
}

fn draw_capture_poopup(frame: &mut Frame, text: &str, character_index: &u16) {
    let area = centered_rect(60, 3, frame.area());

    let block = Block::bordered()
        .title(" diane: ")
        .border_style(Style::default().fg(theme::RED_6E0B22))
        .title_style(Style::default().fg(theme::RED_F53D8F));

    let paragraph = Paragraph::new(Text::from(text))
        .style(Style::default().fg(theme::GRAY_DDD0D6))
        .block(block);

    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
    frame.set_cursor_position(Position::new(area.x + character_index + 1, area.y + 1));
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
