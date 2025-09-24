use std::collections::VecDeque;

use egui::{TextFormat, text::LayoutJob};

use crate::{
    Config,
    kind::{TerminalKind, style::StyleState},
};

#[derive(Default)]
struct Buffer {
    lines: VecDeque<Line>,
}

struct Line {
    text: String,
    sections: Vec<Section>,
}

struct Section {
    style: TextFormat,
    size: usize,
}

pub struct Full {
    buffer: Buffer,

    line: usize,
    column: usize,

    style: StyleState,
}

impl Full {
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
    }

    fn insert(&mut self, c: char, _: TextFormat) {
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        }
    }
}

impl TerminalKind for Full {
    fn new(_: &crate::Config) -> Self {
        Self {
            buffer: Buffer::default(),
            line: 1,
            column: 1,
            style: StyleState::new(),
        }
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
        let mut clone = LayoutJob::default();

        let slow_enabled = cfg.slow_blink_time_seconds != 0.0;
        let fast_enabled = cfg.fast_blink_time_seconds != 0.0;

        let time = ctx.input(|i| i.time);

        let slow_rem = (time % cfg.slow_blink_time_seconds as f64) as f32;
        let fast_rem = (time % cfg.fast_blink_time_seconds as f64) as f32;

        let slow_swap = slow_rem > cfg.slow_blink_time_seconds / 2.0;
        let fast_swap = fast_rem > cfg.fast_blink_time_seconds / 2.0;

        if slow_enabled {
            let half = cfg.slow_blink_time_seconds / 2.0;
            ctx.request_repaint_after_secs(half - (time % half as f64) as f32);
        }
        if fast_enabled {
            let half = cfg.fast_blink_time_seconds / 2.0;
            ctx.request_repaint_after_secs(half - (time % half as f64) as f32);
        }

        for section in &mut clone.sections {
            if section.format.line_height == Some(0.0) && slow_swap
                || section.format.line_height == Some(1.0) && fast_swap
            {
                std::mem::swap(&mut section.format.background, &mut section.format.color);
            }
            section.format.line_height = None;
        }
        clone
    }

    fn clear(&mut self) {
        self.line = 1;
        self.column = 1;
        self.buffer.lines.clear();
    }
}
