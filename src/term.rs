use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Arc,
};

use ansi::AnsiParser;
use egui::text::LayoutJob;

use crate::{Config, state::State};

#[derive(Debug)]
pub struct GenericTerminal<T: ?Sized> {
    state: State,
    pub cfg: Config,
    pub ansi: ansi::AnsiParser<T>,
}

pub type Terminal = GenericTerminal<[u8]>;
pub type StaticTerminal<const B: usize> = GenericTerminal<[u8; B]>;

impl<const C: usize> Deref for GenericTerminal<[u8; C]> {
    type Target = GenericTerminal<[u8]>;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl<const C: usize> DerefMut for GenericTerminal<[u8; C]> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

impl GenericTerminal<[u8]> {
    #[must_use]
    pub fn new_box<const C: usize>(cfg: Config) -> Box<Self> {
        Box::new(GenericTerminal {
            state: State::new(&cfg),
            cfg,
            ansi: AnsiParser::<[u8; C]>::new(),
        })
    }

    #[must_use]
    pub fn new_rc<const C: usize>(cfg: Config) -> Rc<Self> {
        Rc::new(GenericTerminal {
            state: State::new(&cfg),
            cfg,
            ansi: AnsiParser::<[u8; C]>::new(),
        })
    }

    #[must_use]
    pub fn new_arc<const C: usize>(cfg: Config) -> Arc<Self> {
        Arc::new(GenericTerminal {
            state: State::new(&cfg),
            cfg,
            ansi: AnsiParser::<[u8; C]>::new(),
        })
    }

    #[must_use]
    pub fn new_static<const C: usize>(cfg: Config) -> StaticTerminal<C> {
        GenericTerminal {
            state: State::new(&cfg),
            cfg,
            ansi: AnsiParser::<[u8; C]>::new(),
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.state.march(self.ansi.next(*b), &self.cfg);
        }
    }

    pub fn show_bordered(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .inner_margin(2)
            .corner_radius(ui.style().visuals.widgets.noninteractive.corner_radius)
            .fill(self.cfg.bg_default)
            .stroke(ui.style().visuals.window_stroke())
            .show(ui, |ui| {
                egui::ScrollArea::both()
                    .stick_to_bottom(true)
                    .stick_to_right(true)
                    .show(ui, |ui| {
                        ui.label(self.layout(ui.ctx()));
                        ui.allocate_space(ui.available_size());
                    });
            });
    }

    pub fn show_framed(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .inner_margin(2)
            .fill(self.cfg.bg_default)
            .show(ui, |ui| {
                egui::ScrollArea::both()
                    .stick_to_bottom(true)
                    .stick_to_right(true)
                    .show(ui, |ui| {
                        ui.label(self.layout(ui.ctx()));
                        ui.allocate_space(ui.available_size());
                    });
            });
    }

    pub fn clear(&mut self) {
        self.state.clear(&self.cfg);
    }

    #[must_use]
    pub fn layout(&mut self, ctx: &egui::Context) -> LayoutJob {
        self.state.layout(&self.cfg, ctx)
    }
}

impl std::io::Write for GenericTerminal<[u8]> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_bytes(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl std::fmt::Write for GenericTerminal<[u8]> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}
