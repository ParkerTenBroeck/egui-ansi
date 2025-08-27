use eframe::egui;
use egui_ansi::Terminal;

struct MyApp {
    term: Box<Terminal>,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut term = Terminal::new_box::<256>(egui_ansi::Config::DARK);
        _ = print_table(&mut &mut *term);
        Self {
            term,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui Counter Example");

            ui.horizontal(|ui| {
                if ui.button("print").clicked() {
                    _ = print_table(&mut &mut *self.term);
                }
            });
            self.term.show_bordered(ui);
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "egui Counter App",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
    .unwrap();
}

use std::fmt::Write;
const ESC: &str = "\x1b[";

fn print_table(f: &mut impl Write) -> std::fmt::Result {
    // Table header
    writeln!(f, "\nANSI SGR (CSI m) – Common Text/Color Codes\n")?;
    print_rule(f)?;
    writeln!(f, "{:<18} {:<28} Sample", "SGR", "Meaning")?;
    print_rule(f)?;

    // Text attributes
    for (code, name) in attributes() {
        row(f, code, name, "Sample")?;
    }

    print_rule(f)?;
    // 8 standard + 8 bright foreground colors
    for (code, name) in COLOR_FG {
        row(f, code, name, "The quick brown fox")?;
    }

    print_rule(f)?;
    // Bolded foreground colors
    writeln!(f, "Foreground Colors with Bold (SGR 1):\n")?;
    print_rule(f)?;
    writeln!(f, "{:<18} {:<28} Sample", "SGR", "Meaning")?;
    print_rule(f)?;
    for (code, name) in COLOR_FG {
        let combo = format!("1;{}", code);
        let label = format!("Bold {}", name);
        row(f, &combo, &label, "Bold sample")?;
    }

    print_rule(f)?;
    // Faint foreground colors
    writeln!(f, "Foreground Colors with Faint (SGR 2):\n")?;
    print_rule(f)?;
    writeln!(f, "{:<18} {:<28} Sample", "SGR", "Meaning")?;
    print_rule(f)?;
    for (code, name) in COLOR_FG {
        let combo = format!("2;{}", code);
        let label = format!("Faint {}", name);
        row(f, &combo, &label, "Faint sample")?
    }

    print_rule(f)?;
    // 8 standard + 8 bright background colors
    for (code, name) in COLOR_BG {
        row(f, code, name, "  padded sample  ")?;
    }

    print_rule(f)?;
    // Reset/defaults
    for (code, name) in RESETS {
        row(f, code, name, "Back to normal")?;
    }

    print_rule(f)?;

    // 256-color palette (background blocks + plain index text)
    writeln!(
        f,
        "256-color palette (use {}48;5;<n>m for background, {}38;5;<n>m for foreground):\n",
        ESC, ESC
    )?;
    print_palette_256(f)?;
    writeln!(f)?;
    Ok(())
}

// ---------- helpers ----------

fn row(f: &mut impl Write, code: &str, name: &str, sample: &str) -> std::fmt::Result {
    writeln!(
        f,
        "{:<18} {:<28} {}{}m{}{}0m",
        code, name, ESC, code, sample, ESC
    )
}

fn print_rule(f: &mut impl Write) -> std::fmt::Result {
    writeln!(f, "{}", "—".repeat(78))
}

fn attributes() -> Vec<(&'static str, &'static str)> {
    vec![
        ("0", "Reset / Normal"),
        ("1", "Bold / Increased intensity"),
        ("2", "Faint / Decreased intensity"),
        ("3", "Italic"),
        ("4", "Underline"),
        ("5", "Slow blink"),
        ("6", "Rapid blink (rare)"),
        ("7", "Reverse video"),
        ("8", "Conceal / Hidden"),
        ("9", "Crossed-out / Strikethrough"),
        ("21", "Double underline (rare)"),
        ("73", "Super script"),
        ("74", "Sub script"),
    ]
}
const COLOR_FG: &[(&str, &str)] = &[
    ("30", "FG Black"),
    ("31", "FG Red"),
    ("32", "FG Green"),
    ("33", "FG Yellow"),
    ("34", "FG Blue"),
    ("35", "FG Magenta"),
    ("36", "FG Cyan"),
    ("37", "FG White"),
    ("90", "FG Bright Black (Gray)"),
    ("91", "FG Bright Red"),
    ("92", "FG Bright Green"),
    ("93", "FG Bright Yellow"),
    ("94", "FG Bright Blue"),
    ("95", "FG Bright Magenta"),
    ("96", "FG Bright Cyan"),
    ("97", "FG Bright White"),
];

const COLOR_BG: &[(&str, &str)] = &[
    ("40", "BG Black"),
    ("41", "BG Red"),
    ("42", "BG Green"),
    ("43", "BG Yellow"),
    ("44", "BG Blue"),
    ("45", "BG Magenta"),
    ("46", "BG Cyan"),
    ("47", "BG White"),
    ("100", "BG Bright Black (Gray)"),
    ("101", "BG Bright Red"),
    ("102", "BG Bright Green"),
    ("103", "BG Bright Yellow"),
    ("104", "BG Bright Blue"),
    ("105", "BG Bright Magenta"),
    ("106", "BG Bright Cyan"),
    ("107", "BG Bright White"),
];

const RESETS: &[(&str, &str)] = &[
    ("39", "Default foreground color"),
    ("49", "Default background color"),
    ("0", "Full reset (clears all SGR)"),
];

fn print_palette_256(f: &mut impl Write) -> std::fmt::Result {
    // Show background color blocks and the corresponding index in plain text.
    // Group as: 0–15 (system), 16–231 (6×6×6 cube), 232–255 (grayscale)
    writeln!(f, "System (0–15):")?;
    print_palette_range(f, 0, 16, 8)?;

    writeln!(f, "\nColor cube (16–231):")?;
    print_palette_range(f, 16, 216, 6)?;

    writeln!(f, "\nGrayscale (232–255):")?;
    print_palette_range(f, 232, 24, 12)?;

    writeln!(f, "\nTip: use sequences like:")?;
    writeln!(f, "  {}38;5;<n>m  – set 8-bit foreground", ESC)?;
    writeln!(f, "  {}48;5;<n>m  – set 8-bit background", ESC)?;
    writeln!(
        f,
        "  {}38;2;R;G;Bm / {}48;2;R;G;Bm – 24-bit truecolor (if supported)",
        ESC, ESC
    )?;
    Ok(())
}

fn print_palette_range(
    f: &mut impl Write,
    start: u16,
    count: u16,
    per_row: u16,
) -> std::fmt::Result {
    let end = start + count;
    let mut i = start;
    while i < end {
        for j in 0..per_row {
            let n = i + j;
            if n >= end {
                break;
            }
            // colored block
            write!(f, "{}48;5;{}m    {}0m", ESC, n, ESC)?;
            // index in plain text
            write!(f, " {:>3}  ", n)?;
        }
        writeln!(f)?;
        i += per_row;
    }
    Ok(())
}
