use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use tui_markdown::{AlertKind, StyleSheet};

pub const PLUM_261D2A: Color = Color::Rgb(0x26, 0x1d, 0x2a);
pub const PLUM_43364A: Color = Color::Rgb(0x43, 0x36, 0x4a);
pub const PLUM_332839: Color = Color::Rgb(0x33, 0x28, 0x39);
pub const ROSE_FF528B: Color = Color::Rgb(0xff, 0x52, 0x8b);
pub const ROSE_BA2658: Color = Color::Rgb(0xba, 0x26, 0x58);
pub const GOLD_EBD35C: Color = Color::Rgb(0xeb, 0xd3, 0x5c);
pub const GRAY_DDD6E0: Color = Color::Rgb(0xdd, 0xd6, 0xe0);
pub const GRAY_A292AA: Color = Color::Rgb(0xa2, 0x92, 0xaa);
pub const SAGE_81BB94: Color = Color::Rgb(0x81, 0xbb, 0x94);

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

pub const DIANE: Theme = Theme {
    bg: PLUM_261D2A,
    art: ROSE_BA2658,

    border: ROSE_BA2658,
    divider: PLUM_43364A,
    divider_focus: ROSE_BA2658,

    text: GRAY_DDD6E0,
    text_dim: GRAY_A292AA,
    title: ROSE_FF528B,
    hint: GOLD_EBD35C,

    selected_bg: PLUM_43364A,
    selected_bg_dim: PLUM_332839,
    selected_fg: ROSE_FF528B,
};

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
