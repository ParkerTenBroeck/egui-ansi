use crate::Config;
use ansi::{Color, SelectGraphic};
use egui::{Color32, FontId, Stroke, TextFormat};

#[derive(Debug, Default)]
pub enum Underline {
    #[default]
    None,
    Single,
    Double,
}

#[derive(Debug, Default)]
pub enum Weight {
    Faint,
    #[default]
    Normal,
    Bold,
}

#[derive(Debug, Default)]
pub enum Blinking {
    #[default]
    None,
    Slow,
    Fast,
}

#[derive(Debug, Default)]
pub enum Script {
    #[default]
    None,
    Super,
    Sub,
}

#[derive(Debug, Default)]
pub struct StyleState {
    pub fg: Color,
    pub bg: Color,
    pub proportional: bool,
    pub italic: bool,
    pub weight: Weight,
    pub script: Script,
    pub blinking: Blinking,
    pub underline: Underline,
    pub underline_color: Option<Color>,
    pub invert_fg_bg: bool,
    pub strike_through: bool,
    pub conceal: bool,
}

impl StyleState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn march(&mut self, out: ansi::Out<'_>) {
        if let ansi::Out::CSI(csi) = out {
            self.csi(csi.parse())
        }
    }

    pub fn csi(&mut self, csi: ansi::KnownCSI<'_>) {
        if let ansi::KnownCSI::SelectGraphicRendition(gr) = csi {
            for sg in gr {
                self.sg(sg);
            }
        }
    }

    pub fn sg(&mut self, sg: SelectGraphic) {
        match sg {
            SelectGraphic::Reset => {
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
                self.blinking = Blinking::None;
                self.script = Script::None;
            }
            SelectGraphic::Bold => self.weight = Weight::Bold,
            SelectGraphic::Faint => self.weight = Weight::Faint,
            SelectGraphic::NormalIntensity => self.weight = Weight::Normal,
            SelectGraphic::Italic => self.italic = true,
            SelectGraphic::Underline => self.underline = Underline::Single,
            SelectGraphic::SlowBlink => self.blinking = Blinking::Slow,
            SelectGraphic::RapidBlink => self.blinking = Blinking::Fast,
            SelectGraphic::NotBlinking => self.blinking = Blinking::None,
            SelectGraphic::InvertFgBg => self.invert_fg_bg = true,
            SelectGraphic::Conceal => self.conceal = true,
            SelectGraphic::CrossedOut => self.strike_through = true,
            SelectGraphic::Superscript => self.script = Script::Super,
            SelectGraphic::Subscript => self.script = Script::Sub,
            SelectGraphic::NeitherSuperscriptNorSubScript => self.script = Script::None,
            SelectGraphic::PrimaryFont => {}
            SelectGraphic::AlternativeFont(_) => {}
            SelectGraphic::Fraktur => {}
            SelectGraphic::DoublyUnderlined => self.underline = Underline::Double,
            SelectGraphic::NeitherItalicNorBackletter => self.italic = false,
            SelectGraphic::NotUnderlined => self.underline = Underline::None,
            SelectGraphic::ProportionalSpacing => self.proportional = true,
            SelectGraphic::NotInvertedFgBg => self.invert_fg_bg = false,
            SelectGraphic::Reveal => self.conceal = false,
            SelectGraphic::NotCrossedOut => self.strike_through = false,
            SelectGraphic::Fg(color) => self.fg = color,
            SelectGraphic::Bg(color) => self.bg = color,
            SelectGraphic::DisableProportionalSpacing => self.proportional = false,
            SelectGraphic::UnderlineColor(color) => {
                self.underline_color = Some(color);
            }
            _ => {}
        }
    }

    fn color_convert(color: ansi::Color, background: bool, cfg: &Config) -> egui::Color32 {
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

    pub fn format(&self, cfg: &Config) -> TextFormat {
        let mut color = Self::color_convert(self.fg, false, cfg);
        let mut background = Self::color_convert(self.bg, true, cfg);

        if self.invert_fg_bg {
            std::mem::swap(&mut color, &mut background);
        }

        match self.weight {
            Weight::Faint => match if self.invert_fg_bg {
                self.bg.flatten_vga()
            } else {
                self.fg.flatten_vga()
            } {
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
            Weight::Bold => match if self.invert_fg_bg {
                self.bg.flatten_vga()
            } else {
                self.fg.flatten_vga()
            } {
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
            .map_or(color, |c| Self::color_convert(c, false, cfg));

        let font_size = match self.script {
            Script::None => cfg.font_size,
            Script::Super => cfg.superscript_font_size,
            Script::Sub => cfg.subscript_font_size,
        };
        TextFormat {
            font_id: if self.proportional {
                FontId::proportional(font_size)
            } else {
                FontId::monospace(font_size)
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
            expand_bg: cfg.expand_bg,
            extra_letter_spacing: if !self.proportional {
                (cfg.font_size - font_size) / 2.0
            } else {
                0.0
            },
            valign: match self.script {
                Script::None => egui::Align::Center,
                Script::Super => egui::Align::Min,
                Script::Sub => egui::Align::Max,
            },
            line_height: match self.blinking {
                Blinking::None => None,
                Blinking::Slow => Some(0.0),
                Blinking::Fast => Some(1.0),
            },
        }
    }
}
