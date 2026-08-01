use crossterm::terminal;
use ratatui::style::Color;
// This will chage a lot, so i think I prefere to keep the names as
// their hex equivalent untill I settle. Then Ill look for more stylistic names

pub const RED_6E0B22: Color = Color::Rgb(0x6e, 0x0b, 0x22);
pub const RED_C4184F: Color = Color::Rgb(0xc4, 0x18, 0x4f);
pub const RED_F53D8F: Color = Color::Rgb(0xf5, 0x3d, 0x8f);
pub const GRAY_150F13: Color = Color::Rgb(0x15, 0x0f, 0x13);
pub const GRAY_1C141A: Color = Color::Rgb(0x1c, 0x14, 0x1a);
pub const GRAY_231A20: Color = Color::Rgb(0x23, 0x1a, 0x20);
pub const GRAY_DDD0D6: Color = Color::Rgb(0xdd, 0xd0, 0xd6);

const FONT_CORRECTION: f32 = 0.55;
const EDGE_ROW_CHAR: char = '─';
const TOP_LEFT_CORNER_CHAR: char = '┌';
const TOP_RIGHT_CORNER_CHAR: char = '┐';
const BOTTOM_LEFT_CORNER_CHAR: char = '└';
const BOTTOM_RIGHT_CORNER_CHAR: char = '┘';
const EDGE_COLLUMN_CHAR: char = '│';

fn get_terminal_constraints() -> (f32, f32) {
    match terminal::size() {
        Ok((w, h)) => (w.saturating_sub(2) as f32, h.saturating_sub(2) as f32),
        Err(_) => (80.0, 24.0),
    }
}

pub fn texture_pattern() {
    let (max_cols, max_rows) = get_terminal_constraints();

    let diameter_by_width = max_cols;
    let diameter_by_height = max_rows / FONT_CORRECTION;

    let diameter = std::cmp::min(diameter_by_width as i32, diameter_by_height as i32) as f32;

    let cols = diameter as i32;
    let rows = (diameter * FONT_CORRECTION) as i32;

    let radius = diameter / 2.0;
    let center_x = cols as f32 / 2.0;
    let center_y = rows as f32 / 2.0;

    let mut buffer = String::with_capacity((rows * (cols + 1)) as usize);

    for y in 0..rows {
        for x in 0..cols {
            let dx = (x as f32 - center_x) * FONT_CORRECTION;
            let dy = y as f32 - center_y;

            if x == 0 && y == 0 {
                buffer.push(TOP_LEFT_CORNER_CHAR);
            } else if x == cols.saturating_sub_unsigned(1) && y == 0 {
                buffer.push(TOP_RIGHT_CORNER_CHAR);
            } else if y == rows.saturating_sub_unsigned(1) && x == 0 {
                buffer.push(BOTTOM_LEFT_CORNER_CHAR);
            } else if x == cols.saturating_sub_unsigned(1) && y == rows.saturating_sub_unsigned(1) {
                buffer.push(BOTTOM_RIGHT_CORNER_CHAR);
            } else if x == 0 || x == cols.saturating_sub_unsigned(1) {
                buffer.push(EDGE_COLLUMN_CHAR);
            } else if y == 0 || y == rows.saturating_sub_unsigned(1) {
                buffer.push(EDGE_ROW_CHAR);
            } else {
                buffer.push(' ');
            }
            // if (dx * dx) + (dy * dy) <= (radius * FONT_CORRECTION).powi(2) {
            //     buffer.push(BOTTOM_ROW_CHAR);
            // } else {
            //     buffer.push(EDGE_COLLUMN_CHAR);
            // }
        }
        buffer.push('\n');
    }

    println!("{}", buffer);
}
