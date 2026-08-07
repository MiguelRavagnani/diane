use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
//
// This will chage a lot, so i think I prefere to keep the names as
// their hex equivalent untill I settle. Then Ill look for more stylistic names
pub const RED_45060F: Color = Color::Rgb(0x45, 0x06, 0x0f);
pub const RED_6E0B22: Color = Color::Rgb(0x6e, 0x0b, 0x22);
pub const RED_9B1239: Color = Color::Rgb(0x9b, 0x12, 0x39);
pub const RED_C4184F: Color = Color::Rgb(0xc4, 0x18, 0x4f);
pub const RED_DE2467: Color = Color::Rgb(0xde, 0x24, 0x67);
pub const RED_F53D8F: Color = Color::Rgb(0xf5, 0x3d, 0x8f);

pub const GRAY_150F13: Color = Color::Rgb(0x15, 0x0f, 0x13);
pub const GRAY_1C141A: Color = Color::Rgb(0x1c, 0x14, 0x1a);
pub const GRAY_231A20: Color = Color::Rgb(0x23, 0x1a, 0x20);
pub const GRAY_2E1D28: Color = Color::Rgb(0x2e, 0x1d, 0x28);
pub const GRAY_402836: Color = Color::Rgb(0x40, 0x28, 0x36);

pub const GRAY_DDD0D6: Color = Color::Rgb(0xdd, 0xd0, 0xd6);
pub const GRAY_907E88: Color = Color::Rgb(0x90, 0x7e, 0x88);
pub const GRAY_61505A: Color = Color::Rgb(0x61, 0x50, 0x5a);

pub const GRAY_1A0A12: Color = Color::Rgb(0x1a, 0x0a, 0x12);
pub const GRAY_2A0F1A: Color = Color::Rgb(0x2a, 0x0f, 0x1a);

const FILL_CHAR: char = '•';
const NEGATIVE_CHAR: char = ' ';

pub struct Theme {
    pub border: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub spacer: Color,
    pub splash_bg: Color,
    pub splash_fg: Color,
    pub input_text: Color,
    pub info_text: Color,
    pub hint_text: Color,
    pub section_title: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            border: RED_9B1239,
            selected_bg: RED_DE2467,
            selected_fg: GRAY_1C141A,
            spacer: GRAY_402836,
            splash_bg: GRAY_1C141A,
            splash_fg: RED_6E0B22,
            input_text: GRAY_DDD0D6,
            info_text: GRAY_907E88,
            hint_text: RED_F53D8F,
            section_title: RED_DE2467,
        }
    }
}

pub struct BackgroundArt {
    pub amp: isize,
    pub slope: isize,
    pub spacing: isize,
    pub thickness: isize,
    pub ch_fill: char,
    pub ch_negative: char,
    pub theme: Theme,
}

impl Default for BackgroundArt {
    fn default() -> Self {
        Self {
            amp: 3,
            slope: 2,
            spacing: 5,
            thickness: 2,
            ch_fill: FILL_CHAR,
            ch_negative: NEGATIVE_CHAR,
            theme: Theme::default(),
        }
    }
}

impl BackgroundArt {
    fn at(&self, x: usize, y: usize) -> char {
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

impl Widget for BackgroundArt {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let cols = area.width;
        let rows = area.height;

        for y in 0..rows {
            for x in 0..cols {
                let ch = self.at(x as usize, y as usize);
                if let Some(cell) = buf.cell_mut((area.x + x, area.y + y)) {
                    cell.set_char(ch).set_style(
                        Style::new()
                            .fg(self.theme.splash_fg)
                            .add_modifier(Modifier::BOLD),
                    );
                }
            }
        }
    }
}
