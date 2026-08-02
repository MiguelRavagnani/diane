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
const PATTERN: &[u8] = br"//\\";

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

    let mut buffer = String::with_capacity((rows * (cols + 1)) as usize);

    let last_col = cols.saturating_sub(1);
    let last_row = rows.saturating_sub(1);

    for y in 0..rows {
        for x in 0..cols {
            let ch = match (x, y) {
                (0, 0) => TOP_LEFT_CORNER_CHAR,
                (x, 0) if x == last_col => TOP_RIGHT_CORNER_CHAR,
                (0, y) if y == last_row => BOTTOM_LEFT_CORNER_CHAR,
                (x, y) if x == last_col && y == last_row => BOTTOM_RIGHT_CORNER_CHAR,
                (x, _) if x == 0 || x == last_col => EDGE_COLLUMN_CHAR,
                (_, y) if y == 0 || y == last_row => EDGE_ROW_CHAR,
                _ => PATTERN[(x + y) as usize % PATTERN.len()] as char,
            };
            buffer.push(ch);
        }
        buffer.push('\n');
    }
    println!("{}", buffer);
}
