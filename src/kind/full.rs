use std::collections::VecDeque;

use egui::{Color32, FontId, Stroke, TextFormat, text::LayoutJob};

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

#[derive(Default)]
struct CursorPosition {
    line: usize,
    column: usize,
    line_index: usize,
}
impl CursorPosition {
    fn new() -> Self {
        Self {
            line: 1,
            column: 1,
            line_index: 0,
        }
    }
}

pub struct Full {
    buffer: Buffer,

    cursor: CursorPosition,

    show_cusror: bool,

    style: StyleState,
}

impl Full {
    fn csi(&mut self, csi: ansi::KnownCSI<'_>, cfg: &Config) {
        match csi {
            ansi::KnownCSI::SelectGraphicRendition(gr) => {
                for sg in gr {
                    self.style.sg(sg);
                }
            }
            ansi::KnownCSI::CursorRight(count) => {
                for _ in 0..count {
                    self.encounter_char(' ', cfg);
                }
            }
            ansi::KnownCSI::EraseDisplay => self.clear(),
            ansi::KnownCSI::CursorDown(count) => {}
            ansi::KnownCSI::CursorHorizontalAbsolute(h) => {}
            ansi::KnownCSI::CursorLeft(count) => {}
            ansi::KnownCSI::CursorUp(_) => todo!(),
            ansi::KnownCSI::CursorNextLine(_) => todo!(),
            ansi::KnownCSI::CursorPreviousLine(_) => todo!(),
            ansi::KnownCSI::CursorTo { row, col } => todo!(),
            ansi::KnownCSI::HorizontalVerticalPosition { row, col } => todo!(),
            ansi::KnownCSI::CursorPosition => todo!(),
            ansi::KnownCSI::EraseFromCursor => todo!(),
            ansi::KnownCSI::EraseToCursor => todo!(),
            ansi::KnownCSI::EraseScreen => todo!(),
            ansi::KnownCSI::EraseSavedLines => todo!(),
            ansi::KnownCSI::EraseFromCursorToEndOfLine => todo!(),
            ansi::KnownCSI::EraseStartOfLineToCursor => todo!(),
            ansi::KnownCSI::EraseLine => todo!(),
            ansi::KnownCSI::ScrollUp(count) => todo!(),
            ansi::KnownCSI::ScrollDown(count) => todo!(),
            ansi::KnownCSI::AuxPortOn => todo!(),
            ansi::KnownCSI::AuxPortOff => todo!(),
            ansi::KnownCSI::DeviceStatusReport => todo!(),
            ansi::KnownCSI::SaveCurrentCursorPosition => todo!(),
            ansi::KnownCSI::RestoreCurrentCursorPosition => todo!(),
            ansi::KnownCSI::ShowCursor => todo!(),
            ansi::KnownCSI::HideCursor => todo!(),
            ansi::KnownCSI::EnableFocusReporting => todo!(),
            ansi::KnownCSI::DisableFocusReporting => todo!(),
            ansi::KnownCSI::EnableBracketPastingMode => todo!(),
            ansi::KnownCSI::DisableBracketPastingMode => todo!(),
            ansi::KnownCSI::RestoreScreen => todo!(),
            ansi::KnownCSI::SaveScreen => todo!(),
            ansi::KnownCSI::EnableAlternativeBuffer => todo!(),
            ansi::KnownCSI::DisableAlternativeBuffer => todo!(),
            ansi::KnownCSI::ScreenMode(screen_mode) => todo!(),
            ansi::KnownCSI::ResetScreenMode(screen_mode) => todo!(),
            ansi::KnownCSI::SetScrollingRegion { top, bottom } => todo!(),
            ansi::KnownCSI::DeleteLines(_) => todo!(),
            ansi::KnownCSI::InsertLines(_) => todo!(),
            ansi::KnownCSI::CursorLineAbsolute(_) => todo!(),
            ansi::KnownCSI::ReportedCursorPosition { row, col } => todo!(),
            ansi::KnownCSI::ReportCursorPosition => todo!(),
            _ => {}
        }
    }

    fn encounter_char(&mut self, c: char, cfg: &Config) {
        let format = self.style.format(cfg);
        if self.cursor.column > cfg.max_columns && c != '\n' {
            self.insert('\n', format.clone());
        }
        self.insert(c, format);
    }

    fn insert(&mut self, c: char, _: TextFormat) {
        use unicode_width::UnicodeWidthChar;
        let width = c.width().unwrap_or_default();
        self.cursor.column += width;
        self.cursor.line_index += c.len_utf8();
        if c == '\n' {
            self.cursor.line += 1;
            self.cursor.column = 1;
            self.cursor.line_index = 0;
        }
    }
}

impl TerminalKind for Full {
    fn new(_: &crate::Config) -> Self {
        Self {
            buffer: Buffer::default(),
            show_cusror: true,
            cursor: CursorPosition::new(),
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
        let mut layout = LayoutJob::default();

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

        for (line, contents) in self.buffer.lines.iter().enumerate() {
            let line = line + 1;
            let mut offset = 0;
            for section in &contents.sections {
                let end = offset + section.size;
                let mut style = section.style.clone();
                if style.line_height == Some(0.0) && slow_swap
                    || style.line_height == Some(1.0) && fast_swap
                {
                    std::mem::swap(&mut style.background, &mut style.color);
                }

                style.line_height = None;
                layout.append(&contents.text[offset..end], 0.0, style);
                offset = end;
            }
            if line == self.cursor.line && offset == self.cursor.line_index{
                layout.append(" ", 0.0, TextFormat{
                    font_id: FontId::monospace(cfg.font_size),
                    color: Color32::TRANSPARENT,
                    background: cfg.fg_default,
                    ..Default::default()
                });
            }
            if line != self.buffer.lines.len(){
                layout.append("\n", 0.0, TextFormat::simple(FontId::monospace(cfg.font_size), Color32::TRANSPARENT));
            }
        }
        layout
    }

    fn clear(&mut self) {
        self.cursor = CursorPosition::new();
        self.buffer.lines.clear();
    }
}
