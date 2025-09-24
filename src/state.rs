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
enum Blinking {
    None,
    Slow,
    Fast,
}

#[derive(Debug)]
enum Script {
    None,
    Super,
    Sub,
}

#[derive(Debug)]
pub struct State {
    layout: LayoutJob,

    lines: usize,

    fg: Color,
    bg: Color,
    proportional: bool,
    italic: bool,
    weight: Weight,
    script: Script,
    blinking: Blinking,
    underline: Underline,
    underline_color: Option<Color>,
    invert_fg_bg: bool,
    strike_through: bool,
    conceal: bool,

    column: usize,
    line: usize,
    text_index: usize,
    parts_index: usize,

    last_fast_blink: f64,
    last_slow_blink: f64,
}

impl State {
    pub fn new(cfg: &Config) -> Self {
        Self {
            layout: cfg.default_layout(),

            lines: 0,

            fg: Color::Default,
            bg: Color::Default,
            proportional: false,
            italic: false,
            weight: Weight::Normal,
            script: Script::None,
            blinking: Blinking::None,
            underline: Underline::None,
            underline_color: None,
            invert_fg_bg: false,
            strike_through: false,
            conceal: false,

            column: 1,
            line: 1,
            parts_index: 0,
            text_index: 0,

            last_fast_blink: 0.0,
            last_slow_blink: 0.0,
        }
    }

    pub(crate) fn march(&mut self, out: ansi::Out<'_>, cfg: &Config) {
        match out {
            ansi::Out::Data(c) => self.encounter_char(c, cfg),
            ansi::Out::SP => self.encounter_char(' ', cfg),
            ansi::Out::CSI(csi) => self.csi(csi.parse(), cfg),
            ansi::Out::C0(c0) => self.encounter_char(c0 as u8 as char, cfg),
            _ => {}
        }
    }

    fn csi(&mut self, csi: ansi::KnownCSI<'_>, cfg: &Config) {
        match csi {
            ansi::KnownCSI::CursorRight(count) => {
                for _ in 0..count {
                    self.encounter_char(' ', cfg);
                }
            }
            ansi::KnownCSI::CursorNextLine(count) => {
                for _ in 0..count {
                    self.encounter_char('\n', cfg);
                }
            }
            ansi::KnownCSI::EraseDisplay => self.clear(cfg),
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

    fn current_format(&mut self, cfg: &Config) -> TextFormat {
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
            expand_bg: cfg.expand_bg
                + match self.blinking {
                    Blinking::None => 0.0,
                    Blinking::Slow => -0.00001,
                    Blinking::Fast => 0.00001,
                },
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
            line_height: None,
        }
    }

    fn encounter_char(&mut self, c: char, cfg: &Config) {
        let format = self.current_format(cfg);
        // use unicode_width::UnicodeWidthChar;
        // let width = c.width().unwrap_or_default();

        if let Some(last) = self.layout.sections.last_mut()
            && last.format == format
        {
            last.byte_range.end += c.len_utf8();
            self.layout.text.push(c);
        } else {
            self.layout
                .append(c.encode_utf8(&mut [0u8; 4]), 0.0, format);
        }
        if c == '\n' {
            self.lines += 1;
            if self.lines > cfg.max_lines {}
        }
        while self.lines > cfg.max_lines {
            self.delete_line();
        }
    }

    fn delete_line(&mut self) {
        if let Some(at) = self.layout.text.find('\n') {
            let new = self.layout.text.split_off(at + 1);
            let cutoff = self.layout.text.len();
            self.layout.text = new;
            self.layout.sections.retain_mut(|section| {
                if section.byte_range.end <= cutoff {
                    false
                } else {
                    section.byte_range.start = section.byte_range.start.saturating_sub(cutoff);
                    section.byte_range.end = section.byte_range.end.saturating_sub(cutoff);
                    true
                }
            });
            self.lines -= 1;
        } else {
            self.layout.text.clear();
            self.layout.sections.clear();
            self.lines = 0;
        }
    }

    fn blink(&mut self, cfg: &Config, ctx: &egui::Context) {
        let fast_enabled = cfg.fast_blink_time_seconds != 0.0;
        let slow_enabled = cfg.slow_blink_time_seconds != 0.0;
        if !fast_enabled && !slow_enabled {
            return;
        }
        let time = ctx.input(|i| i.time);
        let fast =
            fast_enabled && time - self.last_fast_blink >= cfg.fast_blink_time_seconds as f64;
        let slow =
            slow_enabled && time - self.last_slow_blink >= cfg.slow_blink_time_seconds as f64;
        if fast {
            self.last_fast_blink = time;
        }
        if slow {
            self.last_slow_blink = time;
        }
        if fast_enabled {
            ctx.request_repaint_after_secs(
                cfg.fast_blink_time_seconds - (time - self.last_fast_blink) as f32,
            );
        }
        if slow_enabled {
            ctx.request_repaint_after_secs(
                cfg.slow_blink_time_seconds - (time - self.last_slow_blink) as f32,
            );
        }

        if fast | slow {
            for part in &mut self.layout.sections {
                if part.format.expand_bg < cfg.expand_bg && slow
                    || part.format.expand_bg > cfg.expand_bg && fast
                {
                    std::mem::swap(&mut part.format.background, &mut part.format.color);
                }
            }
        }
    }

    pub(crate) fn layout(&mut self, cfg: &Config, ctx: &egui::Context) -> LayoutJob {
        self.blink(cfg, ctx);
        self.layout.clone()
    }

    pub fn clear(&mut self, cfg: &Config) {
        self.layout = cfg.default_layout();
        self.lines = 0;
    }
}
