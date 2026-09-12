//! MazeLab screen mock. THROWAWAY PROTOTYPE, not the start of the application.
//!
//! Answers issue #6: what does the MazeLab screen look like, and what are the
//! keys? Three structurally different layouts on one static, hard-coded run,
//! switchable with the left and right arrow keys, plus four pages that carry
//! the palette, the keymap, the too-small panel and the footprint report.
//!
//! There is no event loop here in the MazeLab sense: no tick, no step budget,
//! no algorithm. The loop below reads one key and redraws, and it exists only
//! to move between the pages.
//!
//!   cargo run

mod capture;
mod mock;
mod pages;
mod render;
mod theme;
mod variants;

use ratatui::Frame;
use ratatui::crossterm::event::{self, KeyCode};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use variants::Variant;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Screen,
    Palette,
    Keymap,
    TooSmall,
    Footprint,
}

impl Page {
    pub const ALL: [Page; 5] = [
        Page::Screen,
        Page::Palette,
        Page::Keymap,
        Page::TooSmall,
        Page::Footprint,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Page::Screen => "screen",
            Page::Palette => "palette",
            Page::Keymap => "keymap",
            Page::TooSmall => "too small",
            Page::Footprint => "footprint",
        }
    }
}

pub struct Proto {
    pub page: Page,
    pub variant: usize,
    pub ascii: bool,
    pub generating: bool,
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "capture") {
        capture::run(!args.iter().any(|a| a == "plain"));
        return Ok(());
    }

    let mut proto = Proto {
        page: Page::Screen,
        variant: 0,
        ascii: false,
        generating: false,
    };

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| draw(frame, &proto))?;
            let Some(k) = event::read()?.as_key_press_event() else {
                continue;
            };
            match k.code {
                KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                KeyCode::Left => {
                    proto.variant = (proto.variant + Variant::ALL.len() - 1) % Variant::ALL.len();
                }
                KeyCode::Right => {
                    proto.variant = (proto.variant + 1) % Variant::ALL.len();
                }
                KeyCode::Tab => {
                    let i = Page::ALL.iter().position(|p| *p == proto.page).unwrap();
                    proto.page = Page::ALL[(i + 1) % Page::ALL.len()];
                }
                KeyCode::Char('a') => proto.ascii = !proto.ascii,
                KeyCode::Char('g') => proto.generating = !proto.generating,
                KeyCode::Char(c @ '1'..='5') => {
                    proto.page = Page::ALL[c as usize - '1' as usize];
                }
                _ => {}
            }
        }
    })
}

pub fn draw(frame: &mut Frame, proto: &Proto) {
    let area = frame.area();
    // The bottom row belongs to the prototype, never to the design.
    let [body, bar] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
    let variant = Variant::ALL[proto.variant];

    match proto.page {
        Page::Screen => variants::draw(frame, body, variant, proto.ascii, proto.generating),
        Page::Palette => pages::draw_palette(frame, body, proto.ascii),
        Page::Keymap => pages::draw_keymap(frame, body),
        Page::TooSmall => pages::draw_too_small_page(frame, body),
        Page::Footprint => pages::draw_footprint(frame, body, (area.width, area.height)),
    }

    frame.render_widget(switcher(proto, variant, area), bar);
}

fn switcher(proto: &Proto, variant: Variant, area: Rect) -> Paragraph<'static> {
    let tag = Style::new()
        .fg(Color::Black)
        .bg(Color::LightMagenta)
        .add_modifier(Modifier::BOLD);
    let body = Style::new().fg(Color::LightMagenta);
    let dimmed = Style::new().fg(Color::DarkGray);

    let mut spans = vec![
        Span::styled(" PROTOTYPE ", tag),
        Span::styled(format!(" Tab {}  ", proto.page.name()), body),
    ];
    if proto.page == Page::Screen {
        spans.push(Span::styled(
            if area.width < 100 {
                format!("<-/-> {}  ", variant.key())
            } else {
                format!("<-/-> {} {}  ", variant.key(), variant.name())
            },
            body.add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!("g {}  ", if proto.generating { "Generating" } else { "Solved" }),
            body,
        ));
    }
    spans.push(Span::styled(
        format!("a {}  ", if proto.ascii { "--ascii" } else { "unicode" }),
        body,
    ));
    spans.push(Span::styled(
        format!("1-5 pages  q quit  {}x{}", area.width, area.height),
        dimmed,
    ));
    Paragraph::new(Line::from(spans))
}
