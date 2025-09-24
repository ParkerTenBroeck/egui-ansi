use egui::text::LayoutJob;

use crate::Config;

pub mod basic;
pub mod full;
pub mod style;

pub trait TerminalKind {
    fn new(cfg: &Config) -> Self;
    fn march(&mut self, data: ansi::Out<'_>, cfg: &Config);
    fn layout(&mut self, cfg: &Config, ctx: &egui::Context) -> LayoutJob;
    fn clear(&mut self);
}
