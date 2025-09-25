use egui::{
    TextFormat,
    text::{LayoutJob, TextWrapping},
};

use crate::{
    Config,
    kind::{TerminalKind, style::StyleState},
};

pub struct Basic {
    layout: LayoutJob,
    line: usize,
    column: usize,

    style: StyleState,
}

impl Basic {
    fn csi(&mut self, csi: ansi::KnownCSI<'_>, cfg: &Config) {
        match csi {
            ansi::KnownCSI::CursorRight(count) => {
                for _ in 0..count {
                    self.encounter_char(' ', cfg);
                }
            }
            ansi::KnownCSI::EraseDisplay => self.clear(),
            ansi::KnownCSI::SelectGraphicRendition(gr) => {
                for sg in gr {
                    self.style.sg(sg);
                }
            }
            _ => {}
        }
    }

    fn encounter_char(&mut self, c: char, cfg: &Config) {
        let format = self.style.format(cfg);
        use unicode_width::UnicodeWidthChar;
        let width = c.width().unwrap_or_default();
        self.column += width;
        if self.column > cfg.max_columns && c != '\n' {
            self.insert('\n', format.clone());
        }
        self.insert(c, format);

        while self.line > cfg.max_rows {
            self.delete_line();
        }
    }

    fn insert(&mut self, c: char, format: TextFormat) {
        if let Some(last) = self.layout.sections.last_mut()
            && last.format == format
        {
            last.byte_range.end += c.len_utf8();
            self.layout.text.push(c);
        } else {
            let spacing = if self.style.proportional {
                0.0
            } else {
                format.extra_letter_spacing
            };
            self.layout
                .append(c.encode_utf8(&mut [0u8; 4]), spacing, format);
        }
        if c == '\n' {
            self.line += 1;
            self.column = 1;
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
            self.line -= 1;
        } else {
            self.clear();
        }
    }
}

impl TerminalKind for Basic {
    fn new(_: &crate::Config) -> Self {
        let mut me = Self {
            layout: Default::default(),
            line: 1,
            column: 1,
            style: StyleState::new(),
        };
        me.clear();
        me
    }

    fn march(&mut self, out: ansi::Out<'_>, cfg: &Config) {
        match out {
            ansi::Out::Data(c) => self.encounter_char(c, cfg),
            ansi::Out::SP => self.encounter_char(' ', cfg),
            ansi::Out::CSI(csi) => self.csi(csi.parse(), cfg),
            ansi::Out::C0(c0) => self.encounter_char(c0 as u8 as char, cfg),
            _ => {}
        }
    }

    fn layout(&mut self, cfg: &crate::Config, ctx: &egui::Context) -> LayoutJob {
        let mut clone = self.layout.clone();

        let slow_enabled = cfg.slow_blink_time_seconds != 0.0;
        let fast_enabled = cfg.fast_blink_time_seconds != 0.0;

        let time = ctx.input(|i| i.time);

        let slow_rem = (time % cfg.slow_blink_time_seconds as f64) as f32;
        let fast_rem = (time % cfg.fast_blink_time_seconds as f64) as f32;

        let slow_swap = slow_enabled && slow_rem > cfg.slow_blink_time_seconds / 2.0;
        let fast_swap = fast_enabled && fast_rem > cfg.fast_blink_time_seconds / 2.0;

        let mut slow = false;
        let mut fast = false;

        for section in &mut clone.sections {
            if section.format.line_height == Some(0.0) {
                slow = true;
                if slow_swap {
                    std::mem::swap(&mut section.format.background, &mut section.format.color);
                }
            } else if section.format.line_height == Some(1.0) {
                fast = true;
                if fast_swap {
                    std::mem::swap(&mut section.format.background, &mut section.format.color);
                }
            }
            section.format.line_height = None;
        }
        if slow {
            let half = cfg.slow_blink_time_seconds / 2.0;
            ctx.request_repaint_after_secs(half - (time % half as f64) as f32);
        }
        if fast {
            let half = cfg.fast_blink_time_seconds / 2.0;
            ctx.request_repaint_after_secs(half - (time % half as f64) as f32);
        }
        clone
    }

    fn clear(&mut self) {
        self.line = 1;
        self.column = 1;
        self.layout = LayoutJob::default();
        self.layout.wrap = TextWrapping::no_max_width();
    }
}
