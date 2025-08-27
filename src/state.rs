use crate::Config;
use ansi::{Color, SelectGraphic};
use egui::{Color32, FontId, Stroke, TextFormat, text::LayoutJob};

#[derive(Debug)]
enum Underline {
    None,
    Single,
    Double,
}

#[derive(Debug)]
enum Weight {
    Faint,
    Normal,
    Bold,
}

#[derive(Debug)]
pub struct State {
    layout: LayoutJob,

    fg: Color,
    bg: Color,
    proportional: bool,
    italic: bool,
    weight: Weight,
    underline: Underline,
    underline_color: Option<Color>,
    invert_fg_bg: bool,
    strike_through: bool,
    conceal: bool,
}

impl State {
    pub fn new(cfg: &Config) -> Self {
        Self {
            layout: cfg.default_layout(),
            fg: Color::Default,
            bg: Color::Default,
            proportional: false,
            italic: false,
            weight: Weight::Normal,
            underline: Underline::None,
            underline_color: None,
            invert_fg_bg: false,
            strike_through: false,
            conceal: false,
        }
    }

    pub(crate) fn march(&mut self, out: ansi::Out<'_>, cfg: &Config) {
        match out {
            ansi::Out::Data(c) => self.append_char(c, cfg),
            ansi::Out::SP => self.append_char(' ', cfg),
            ansi::Out::CSI(csi) => self.csi(csi.parse(), cfg),
            ansi::Out::C0(c0) => self.append_char(c0 as u8 as char, cfg),
            _ => {}
        }
    }

    fn csi(&mut self, csi: ansi::KnownCSI<'_>, cfg: &Config) {
        match csi {
            ansi::KnownCSI::CursorRight(count) => {
                for _ in 0..count {
                    self.append_char(' ', cfg)
                }
            }
            ansi::KnownCSI::CursorNextLine(count) => {
                for _ in 0..count {
                    self.append_char('\n', cfg)
                }
            }
            ansi::KnownCSI::EraseDisplay => self.layout = cfg.default_layout(),
            ansi::KnownCSI::SelectGraphicRendition(gr) => {
                for sg in gr {
                    self.sg(sg);
                }
            }
            _ => {}
        }
    }

    fn sg(&mut self, sg: SelectGraphic) {
        match sg {
            ansi::SelectGraphic::Reset => {
                self.fg = Color::Default;
                self.bg = Color::Default;
                self.proportional = false;
                self.italic = false;
                self.weight = Weight::Normal;
                self.underline = Underline::None;
                self.underline_color = None;
                self.invert_fg_bg = false;
                self.strike_through = false;
                self.conceal = false;
            }

            ansi::SelectGraphic::Bold => self.weight = Weight::Bold,
            ansi::SelectGraphic::Faint => self.weight = Weight::Faint,
            ansi::SelectGraphic::NormalIntensity => self.weight = Weight::Normal,

            ansi::SelectGraphic::Italic => self.italic = true,
            ansi::SelectGraphic::Underline => self.underline = Underline::Single,
            ansi::SelectGraphic::SlowBlink => {}
            ansi::SelectGraphic::RapidBlink => {}
            ansi::SelectGraphic::InvertFgBg => self.invert_fg_bg = true,
            ansi::SelectGraphic::Conceal => self.conceal = true,
            ansi::SelectGraphic::CrossedOut => self.strike_through = true,
            ansi::SelectGraphic::PrimaryFont => {}
            ansi::SelectGraphic::AlternativeFont(_) => {}
            ansi::SelectGraphic::Fraktur => {}
            ansi::SelectGraphic::DoublyUnderlined => self.underline = Underline::Double,
            ansi::SelectGraphic::NeitherItalicNorBackletter => self.italic = false,
            ansi::SelectGraphic::NotUnderlined => self.underline = Underline::None,
            ansi::SelectGraphic::NotBlinking => {}
            ansi::SelectGraphic::ProportionalSpacing => self.proportional = true,
            ansi::SelectGraphic::NotInvertedFgBg => self.invert_fg_bg = false,
            ansi::SelectGraphic::Reveal => self.conceal = false,
            ansi::SelectGraphic::NotCrossedOut => self.strike_through = false,
            ansi::SelectGraphic::Fg(color) => self.fg = color,
            ansi::SelectGraphic::Bg(color) => self.bg = color,
            ansi::SelectGraphic::DisableProportionalSpacing => self.proportional = false,
            ansi::SelectGraphic::UnderlineColor(color) => {
                self.underline_color = Some(color);
            }
            _ => {}
        }
    }

    fn color_convert(color: ansi::Color, background: bool, cfg: &Config) -> Color32 {
        match color.flatten_vga() {
            ansi::Color::Default => {
                if background {
                    cfg.black
                } else {
                    cfg.white
                }
            }
            ansi::Color::Black => cfg.black,
            ansi::Color::Red => cfg.red,
            ansi::Color::Green => cfg.green,
            ansi::Color::Yellow => cfg.yellow,
            ansi::Color::Blue => cfg.blue,
            ansi::Color::Magenta => cfg.magenta,
            ansi::Color::Cyan => cfg.cyan,
            ansi::Color::White => cfg.white,
            ansi::Color::BrightBlack => cfg.bright_black,
            ansi::Color::BrightRed => cfg.bright_red,
            ansi::Color::BrightGreen => cfg.bright_green,
            ansi::Color::BrightYellow => cfg.bright_yellow,
            ansi::Color::BrightBlue => cfg.bright_blue,
            ansi::Color::BrightMagenta => cfg.bright_magenta,
            ansi::Color::BrightCyan => cfg.bright_cyan,
            ansi::Color::BrightWhite => cfg.bright_white,
            ansi::Color::RGB(rgb) => egui::Color32::from_rgb(rgb.r, rgb.g, rgb.b),
            _ => egui::Color32::PLACEHOLDER,
        }
    }

    fn append_char(&mut self, c: char, cfg: &Config) {
        let mut color = Self::color_convert(self.fg, false, cfg);
        let mut background = Self::color_convert(self.bg, true, cfg);

        if self.invert_fg_bg {
            std::mem::swap(&mut color, &mut background);
        }

        match self.weight {
            Weight::Faint => match if self.invert_fg_bg { self.bg.flatten_vga() } else { self.fg.flatten_vga() } {
                Color::BrightBlack => color = cfg.black,
                Color::BrightRed => color = cfg.red,
                Color::BrightGreen => color = cfg.green,
                Color::BrightYellow => color = cfg.yellow,
                Color::BrightBlue => color = cfg.blue,
                Color::BrightMagenta => color = cfg.magenta,
                Color::BrightCyan => color = cfg.cyan,
                Color::BrightWhite => color = cfg.white,
                _ => color = color.gamma_multiply(0.5),
            },
            Weight::Normal => {}
            Weight::Bold => match if self.invert_fg_bg { self.bg.flatten_vga() } else { self.fg.flatten_vga() } {
                Color::Black => color = cfg.bright_black,
                Color::Red => color = cfg.bright_red,
                Color::Green => color = cfg.bright_green,
                Color::Yellow => color = cfg.bright_yellow,
                Color::Blue => color = cfg.bright_blue,
                Color::Magenta => color = cfg.bright_magenta,
                Color::Cyan => color = cfg.bright_cyan,
                Color::White => color = cfg.bright_white,
                _ => color = color.gamma_multiply(1.5),
            },
        }

        if self.conceal {
            color = Color32::TRANSPARENT;
        }

        let underline = self
            .underline_color
            .map(|c| Self::color_convert(c, false, cfg))
            .unwrap_or(color);
        let format = TextFormat {
            font_id: if self.proportional {
                FontId::proportional(cfg.font_size)
            } else {
                FontId::monospace(cfg.font_size)
            },
            color,
            background,
            italics: self.italic,
            underline: match self.underline {
                Underline::None => Stroke::NONE,
                Underline::Single => Stroke::new(cfg.underline_width, underline),
                Underline::Double => Stroke::new(cfg.double_underline_width, underline),
            },
            strikethrough: if self.strike_through {
                Stroke::new(cfg.strike_through_width, color)
            } else {
                Stroke::NONE
            },

            ..Default::default()
        };
        if let Some(last) = self.layout.sections.last_mut()
            && last.format == format
        {
            last.byte_range.end += c.len_utf8();
            self.layout.text.push(c);
        } else {
            self.layout
                .append(c.encode_utf8(&mut [0u8; 4]), 0.0, format);
        }
    }

    pub(crate) fn layout(&self) -> LayoutJob {
        self.layout.clone()
    }
}
