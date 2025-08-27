use egui::{Color32, text::LayoutJob};

#[derive(Debug)]
pub struct Config {
    pub font_size: f32,
    pub subscript_font_size: f32,
    pub superscript_font_size: f32,

    pub max_lines: usize,

    pub expand_bg: f32,

    pub strike_through_width: f32,
    pub underline_width: f32,
    pub double_underline_width: f32,

    pub fg_default: Color32,
    pub bg_default: Color32,

    pub black: Color32,
    pub red: Color32,
    pub green: Color32,
    pub yellow: Color32,
    pub blue: Color32,
    pub magenta: Color32,
    pub cyan: Color32,
    pub white: Color32,

    pub bright_black: Color32,
    pub bright_red: Color32,
    pub bright_green: Color32,
    pub bright_yellow: Color32,
    pub bright_blue: Color32,
    pub bright_magenta: Color32,
    pub bright_cyan: Color32,
    pub bright_white: Color32,
}

const fn color(raw: u32) -> Color32 {
    Color32::from_rgb((raw >> 16) as u8, (raw >> 8) as u8, raw as u8)
}

impl Config {
    pub const DARK: Self = Self {
        font_size: 14.0,
        subscript_font_size: 10.0,
        superscript_font_size: 10.0,

        max_lines: 1000,
        expand_bg: 0.0,

        strike_through_width: 1.0,
        underline_width: 2.0,
        double_underline_width: 4.0,

        fg_default: Color32::from_gray(0xcc),
        bg_default: Color32::from_gray(0x18),

        black: Color32::from_gray(0x18),
        red: color(0xe74856),
        green: color(0x16c60c),
        yellow: color(0xf9f1a5),
        blue: color(0x3b78ff),
        magenta: color(0xb4009e),
        cyan: color(0x61d6d6),
        white: Color32::from_gray(0xcc),

        bright_black: Color32::from_rgb(0x66, 0x66, 0x66),
        bright_red: color(0xc50f1f),
        bright_green: color(0x13a10e),
        bright_yellow: color(0xc19c00),
        bright_blue: color(0x0037da),
        bright_magenta: color(0x881798),
        bright_cyan: color(0x3a96dd),
        bright_white: Color32::from_gray(0xf2),
    };

    pub fn default_layout(&self) -> LayoutJob {
        LayoutJob::default()
    }
}
