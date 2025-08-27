use ansi::AnsiParser;
use egui::text::LayoutJob;

use crate::{Config, state::State};

#[derive(Debug)]
pub struct Terminal {
    state: State,
    cfg: Config,
    ansi: ansi::SizedAnsiParser<256>,
}

impl Terminal {
    pub fn new(cfg: Config) -> Self {
        Self {
            ansi: AnsiParser::new(),
            state: State::new(&cfg),
            cfg,
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.state.march(self.ansi.next(*b), &self.cfg);
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .inner_margin(2)
            .corner_radius(ui.style().visuals.widgets.noninteractive.corner_radius)
            .fill(self.cfg.bg_default)
            .stroke(ui.style().visuals.window_stroke())
            .show(ui, |ui| {
                egui::ScrollArea::both()
                    .stick_to_bottom(true)
                    .stick_to_right(true)
                    .show(ui, |ui| ui.label(self.layout()));
            });
    }

    pub fn layout(&self) -> LayoutJob {
        self.state.layout()
    }
}

impl std::io::Write for Terminal {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_bytes(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl std::fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}
