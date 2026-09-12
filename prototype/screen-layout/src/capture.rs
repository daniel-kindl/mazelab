//! Render every view to a TestBackend and print it, so the mock can be looked
//! at without running it. THROWAWAY.
//!
//!   cargo run -- capture plain    layout check, no colour
//!   cargo run -- capture ansi     colour, for a terminal or a pipe to a file

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier};

use crate::{Page, Proto, draw};
use crate::variants::Variant;

pub fn run(ansi: bool) {
    for (cols, rows) in [(120u16, 30u16), (80, 24)] {
        for (page, variant, generating) in views() {
            let proto = Proto { page, variant, ascii: false, generating };
            let title = format!(
                "{}  {}  {}x{}",
                page.name(),
                if page == Page::Screen {
                    format!(
                        "variant {} ({}), {}",
                        Variant::ALL[variant].key(),
                        Variant::ALL[variant].name(),
                        if generating { "Generating" } else { "Solved" }
                    )
                } else {
                    String::new()
                },
                cols,
                rows
            );
            print_view(&title, &proto, cols, rows, ansi);
        }
    }
    // The --ascii escape hatch, one view only.
    let proto = Proto { page: Page::Screen, variant: 0, ascii: true, generating: false };
    print_view("screen  variant A, --ascii  120x30", &proto, 120, 30, ansi);
}

fn views() -> Vec<(Page, usize, bool)> {
    let mut v = Vec::new();
    for i in 0..Variant::ALL.len() {
        v.push((Page::Screen, i, false));
    }
    v.push((Page::Screen, 0, true));
    v.push((Page::Palette, 0, false));
    v.push((Page::Keymap, 0, false));
    v.push((Page::TooSmall, 0, false));
    v.push((Page::Footprint, 0, false));
    v
}

fn print_view(title: &str, proto: &Proto, cols: u16, rows: u16, ansi: bool) {
    let mut terminal = Terminal::new(TestBackend::new(cols, rows)).unwrap();
    terminal.draw(|frame| draw(frame, proto)).unwrap();
    let buffer = terminal.backend().buffer().clone();

    println!();
    println!("=== {title} {}", "=".repeat(78usize.saturating_sub(title.len())));
    println!("+{}+", "-".repeat(cols as usize));
    for line in buffer_lines(&buffer, cols, rows, ansi) {
        println!("|{line}|");
    }
    println!("+{}+", "-".repeat(cols as usize));
}

fn buffer_lines(buffer: &Buffer, cols: u16, rows: u16, ansi: bool) -> Vec<String> {
    (0..rows)
        .map(|y| {
            let mut out = String::new();
            let mut open = false;
            for x in 0..cols {
                let cell = &buffer[(x, y)];
                if ansi {
                    let mut codes = Vec::new();
                    if cell.modifier.contains(Modifier::BOLD) {
                        codes.push("1".to_string());
                    }
                    if let Some(c) = sgr(cell.fg, false) {
                        codes.push(c);
                    }
                    if let Some(c) = sgr(cell.bg, true) {
                        codes.push(c);
                    }
                    if !codes.is_empty() {
                        out.push_str(&format!("\x1b[0;{}m", codes.join(";")));
                        open = true;
                    } else if open {
                        out.push_str("\x1b[0m");
                        open = false;
                    }
                }
                out.push_str(cell.symbol());
            }
            if open {
                out.push_str("\x1b[0m");
            }
            out
        })
        .collect()
}

fn sgr(color: Color, bg: bool) -> Option<String> {
    let base = match color {
        Color::Reset => return None,
        Color::Black => 30,
        Color::Red => 31,
        Color::Green => 32,
        Color::Yellow => 33,
        Color::Blue => 34,
        Color::Magenta => 35,
        Color::Cyan => 36,
        Color::Gray => 37,
        Color::DarkGray => 90,
        Color::LightRed => 91,
        Color::LightGreen => 92,
        Color::LightYellow => 93,
        Color::LightBlue => 94,
        Color::LightMagenta => 95,
        Color::LightCyan => 96,
        Color::White => 97,
        _ => return None,
    };
    Some((base + if bg { 10 } else { 0 }).to_string())
}
