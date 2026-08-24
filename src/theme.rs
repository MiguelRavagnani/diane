use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use tui_markdown::{AlertKind, StyleSheet};
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

pub const TITLE: &str = "⢀⣴⣄⢴⣷⣄ DIANE:";

const FILL_CHAR: char = '∙';
const NEGATIVE_CHAR: char = ' ';

#[derive(Clone, Copy)]
pub struct Theme {
    pub bg: Color,
    pub art: Color,

    pub border: Color,
    pub divider: Color,
    pub divider_focus: Color,

    pub text: Color,
    pub text_dim: Color,
    pub title: Color,
    pub hint: Color,

    pub selected_bg: Color,
    pub selected_bg_dim: Color,
    pub selected_fg: Color,
}

impl StyleSheet for Theme {
    fn heading(&self, level: u8) -> Style {
        match level {
            1 => Style::default()
                .bg(self.selected_bg)
                .fg(self.selected_fg)
                .add_modifier(Modifier::BOLD),
            2 => Style::default().fg(self.title).add_modifier(Modifier::BOLD),
            3 => Style::default()
                .fg(self.title)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            _ => Style::default()
                .fg(self.hint)
                .add_modifier(Modifier::ITALIC),
        }
    }

    fn code(&self) -> Style {
        Style::default().fg(self.hint).bg(self.selected_bg_dim)
    }

    fn link(&self) -> Style {
        Style::default()
            .fg(self.hint)
            .add_modifier(Modifier::UNDERLINED)
    }

    fn blockquote(&self) -> Style {
        Style::default()
            .fg(self.text_dim)
            .add_modifier(Modifier::ITALIC)
    }

    fn heading_meta(&self) -> Style {
        Style::default().fg(self.text_dim)
    }

    fn metadata_block(&self) -> Style {
        Style::default().fg(self.text_dim)
    }

    fn html(&self) -> Style {
        Style::default().fg(self.text_dim)
    }

    fn math_inline(&self) -> Style {
        Style::default()
            .fg(self.hint)
            .add_modifier(Modifier::ITALIC)
    }

    fn math_display(&self) -> Style {
        Style::default().fg(self.hint)
    }

    fn footnote_ref(&self) -> Style {
        Style::default()
            .fg(self.text_dim)
            .add_modifier(Modifier::ITALIC)
    }

    fn footnote_def(&self) -> Style {
        Style::default().fg(self.text_dim)
    }

    fn definition_term(&self) -> Style {
        Style::default().fg(self.text).add_modifier(Modifier::BOLD)
    }

    fn definition_description(&self) -> Style {
        Style::default().fg(self.text_dim)
    }

    fn alert(&self, kind: AlertKind) -> Style {
        match kind {
            AlertKind::Note | AlertKind::Tip => Style::default().fg(self.text_dim),
            AlertKind::Important => Style::default().fg(self.title),
            AlertKind::Warning | AlertKind::Caution => Style::default().fg(self.hint),
        }
    }

    fn table_header(&self) -> Style {
        Style::default().fg(self.title).add_modifier(Modifier::BOLD)
    }

    fn table_cell(&self) -> Style {
        Style::default().fg(self.text)
    }

    fn table_border(&self) -> Style {
        Style::default().fg(self.divider)
    }

    fn image_alt(&self) -> Style {
        Style::default()
            .fg(self.text_dim)
            .add_modifier(Modifier::ITALIC)
    }
}

pub const DIANE: Theme = Theme {
    bg: GRAY_1C141A,
    art: RED_6E0B22,

    border: RED_9B1239,
    divider: GRAY_402836,
    divider_focus: RED_6E0B22,

    text: GRAY_DDD0D6,
    text_dim: GRAY_907E88,
    title: RED_DE2467,
    hint: RED_F53D8F,

    selected_bg: RED_DE2467,
    selected_bg_dim: GRAY_402836,
    selected_fg: GRAY_1C141A,
};

impl Default for Theme {
    fn default() -> Self {
        DIANE
    }
}

impl Theme {
    pub fn named(name: &str) -> Self {
        match name {
            "default" => DIANE,
            _ => DIANE,
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
    pub art: Color,
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
            art: DIANE.art,
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
                    cell.set_char(ch)
                        .set_style(Style::new().fg(self.art).add_modifier(Modifier::BOLD));
                }
            }
        }
    }
}
