use crossterm::terminal;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
// This will chage a lot, so i think I prefere to keep the names as
// their hex equivalent untill I settle. Then Ill look for more stylistic names

pub const RED_6E0B22: Color = Color::Rgb(0x6e, 0x0b, 0x22);
pub const RED_C4184F: Color = Color::Rgb(0xc4, 0x18, 0x4f);
pub const RED_F53D8F: Color = Color::Rgb(0xf5, 0x3d, 0x8f);
pub const GRAY_150F13: Color = Color::Rgb(0x15, 0x0f, 0x13);
pub const GRAY_1C141A: Color = Color::Rgb(0x1c, 0x14, 0x1a);
pub const GRAY_231A20: Color = Color::Rgb(0x23, 0x1a, 0x20);
pub const GRAY_DDD0D6: Color = Color::Rgb(0xdd, 0xd0, 0xd6);

const EDGE_ROW_CHAR: char = '─';
const TOP_LEFT_CORNER_CHAR: char = '┌';
const TOP_RIGHT_CORNER_CHAR: char = '┐';
const BOTTOM_LEFT_CORNER_CHAR: char = '└';
const BOTTOM_RIGHT_CORNER_CHAR: char = '┘';
const EDGE_COLLUMN_CHAR: char = '│';
const FILL_CHAR: char = '•';
const NEGATIVE_CHAR: char = ' ';

#[derive(Clone, Copy)]
struct Chevron {
    pub amp: isize,
    pub slope: isize,
    pub spacing: isize,
    pub thickness: isize,
    ch_fill: char,
    ch_negative: char,
}

impl Default for Chevron {
    fn default() -> Self {
        Self {
            amp: 3,
            slope: 2,
            spacing: 5,
            thickness: 2,
            ch_fill: FILL_CHAR,
            ch_negative: NEGATIVE_CHAR,
        }
    }
}

impl Chevron {
    pub fn at(&self, x: usize, y: usize) -> char {
        let (x, y) = (x as isize, y as isize);

        let half = self.amp * self.slope;
        let t = x.rem_euclid(2 * half);
        let wave = (t - half).abs();

        let v = (y * self.slope - wave).rem_euclid(self.spacing * self.slope);

        if v < self.thickness * self.slope {
            return self.ch_fill;
        };
        self.ch_negative
    }
}

pub struct BackgroundArt {}

impl Widget for BackgroundArt {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let cols = area.width;
        let rows = area.height;
        let last_col = cols.saturating_sub(1);
        let last_row = rows.saturating_sub(1);

        let chevron = Chevron {
            amp: 3,
            slope: 2,
            spacing: 7,
            thickness: 4,
            ..Default::default()
        };

        for y in 0..rows {
            for x in 0..cols {
                let ch = match (x, y) {
                    (0, 0) => TOP_LEFT_CORNER_CHAR,
                    (x, 0) if x == last_col => TOP_RIGHT_CORNER_CHAR,
                    (0, y) if y == last_row => BOTTOM_LEFT_CORNER_CHAR,
                    (x, y) if x == last_col && y == last_row => BOTTOM_RIGHT_CORNER_CHAR,
                    (x, _) if x == 0 || x == last_col => EDGE_COLLUMN_CHAR,
                    (_, y) if y == 0 || y == last_row => EDGE_ROW_CHAR,
                    _ => chevron.at(x as usize, y as usize),
                };
                if let Some(cell) = buf.cell_mut((area.x + x, area.y + y)) {
                    cell.set_char(ch)
                        .set_style(Style::new().fg(RED_6E0B22).add_modifier(Modifier::DIM));
                }
            }
        }
    }
}
