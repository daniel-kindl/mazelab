//! The non-layout pages: palette, keymap, too-small, footprint. THROWAWAY.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::render::MAZE_COLS;
use crate::render::MAZE_ROWS;
use crate::theme::{self, gen_look, solver_look};
use crate::variants::Variant;

// ------------------------------------------------------------------ palette

/// Decision 16 buys theme inheritance and pays for it with no contrast
/// guarantee. This page shows the same palette on a forced dark ground, a
/// forced light ground, and the ground the user actually has, so the failure
/// is visible rather than argued about.
pub fn draw_palette(frame: &mut Frame, area: Rect, ascii: bool) {
    let [top, bottom] =
        Layout::vertical([Constraint::Fill(3), Constraint::Length(9)]).areas(area);
    // Two boxes side by side need about 100 columns; below that, stack them.
    let [solver, generation] = if top.width < 100 {
        Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(top)
    } else {
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(top)
    };

    let header = Line::from(vec![
        Span::styled(format!("{:<18}", "state"), Style::new().fg(Color::Gray)),
        Span::styled(format!("{:<10}", "terminal"), Style::new().fg(Color::DarkGray)),
        Span::styled(format!("{:<8}", "dark"), Style::new().fg(Color::DarkGray)),
        Span::styled(format!("{:<8}", "light"), Style::new().fg(Color::DarkGray)),
        Span::styled("colour", Style::new().fg(Color::DarkGray)),
    ]);

    let mut solver_lines = vec![header.clone(), Line::from("")];
    for (state, name) in theme::SOLVER_STATES {
        let look = solver_look(*state, ascii);
        solver_lines.push(swatch_row(name, look));
    }
    frame.render_widget(
        Paragraph::new(solver_lines).block(Block::bordered().title(" solver cell state ")),
        solver,
    );

    let mut gen_lines = vec![header, Line::from("")];
    for (state, name) in theme::GEN_STATES {
        let look = gen_look(*state, ascii);
        gen_lines.push(swatch_row(name, look));
    }
    gen_lines.push(Line::from(""));
    gen_lines.push(Line::from(Span::styled(
        "Priority resolves to a cell state,",
        Style::new().fg(Color::DarkGray),
    )));
    gen_lines.push(Line::from(Span::styled(
        "never straight to a colour.",
        Style::new().fg(Color::DarkGray),
    )));
    frame.render_widget(
        Paragraph::new(gen_lines).block(Block::bordered().title(" generation cell state ")),
        generation,
    );

    frame.render_widget(
        Paragraph::new(ansi_rows()).block(Block::bordered().title(
            " the 16 named colours, on a dark ground and a light one ",
        )),
        bottom,
    );
}

fn swatch_row(name: &'static str, look: theme::Look) -> Line<'static> {
    let colour_name = format!("{:?}", look.color);
    let bracket = Style::new().fg(Color::DarkGray);
    let cell = |bg: Option<Color>| {
        let st = match bg {
            Some(b) => look.style().bg(b),
            None => look.style(),
        };
        vec![
            Span::styled("[", bracket),
            Span::styled(look.glyph, st),
            Span::styled("]", bracket),
        ]
    };
    let mut spans = vec![Span::styled(format!("{name:<18}"), Style::new())];
    spans.extend(cell(None));
    spans.push(Span::raw("      "));
    spans.extend(cell(Some(Color::Black)));
    spans.push(Span::raw("    "));
    spans.extend(cell(Some(Color::White)));
    spans.push(Span::raw("    "));
    spans.push(Span::styled(colour_name, Style::new().fg(Color::DarkGray)));
    Line::from(spans)
}

fn ansi_rows() -> Vec<Line<'static>> {
    let chunk = |bg: Color, label: &'static str| {
        let mut spans = vec![Span::styled(
            format!("{label:<8}"),
            Style::new().fg(Color::Gray),
        )];
        for (c, _) in theme::ANSI_16 {
            spans.push(Span::styled("  \u{2588}\u{2588}  ", Style::new().fg(*c).bg(bg)));
        }
        Line::from(spans)
    };
    let names = {
        let mut spans = vec![Span::raw(format!("{:<8}", ""))];
        for short in theme::ANSI_16_SHORT {
            spans.push(Span::styled(
                format!("{short:<6}"),
                Style::new().fg(Color::DarkGray),
            ));
        }
        Line::from(spans)
    };
    vec![
        chunk(Color::Black, "dark"),
        chunk(Color::White, "light"),
        names,
        Line::from(""),
        Line::from(Span::styled(
            "Yellow (ANSI 3, crossterm's DarkYellow) is the documented offender: it is brown on",
            Style::new().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "some themes and near-invisible on others. The palette above keeps it out of the maze.",
            Style::new().fg(Color::Gray),
        )),
    ]
}

// ------------------------------------------------------------------- keymap

pub struct Binding {
    pub keys: &'static str,
    pub action: &'static str,
    pub note: &'static str,
}

pub const KEYMAP: &[Binding] = &[
    Binding { keys: "g", action: "generate", note: "animated, from the current seed" },
    Binding { keys: "G", action: "generate instantly", note: "skips the animation, decision 10" },
    Binding { keys: "s", action: "solve", note: "runs the selected solver" },
    Binding { keys: "Space", action: "pause / resume", note: "Generating <-> Paused, Solving <-> Paused" },
    Binding { keys: ".", action: "single step", note: "exactly one step, whatever the speed" },
    Binding { keys: "+  -", action: "speed up / down", note: "one rung of the ladder" },
    Binding { keys: "1  2", action: "choose generator", note: "1 Recursive Backtracker, 2 Randomized Prim" },
    Binding { keys: "3  4  5", action: "choose solver", note: "3 DFS, 4 BFS, 5 A*; Solved -> Ready" },
    Binding { keys: "Tab", action: "next solver", note: "shortcut for 3-5" },
    Binding { keys: "<-  ->", action: "maze narrower / wider", note: "decision 9, explicit size" },
    Binding { keys: "v  ^", action: "maze shorter / taller", note: "Down and Up arrows" },
    Binding { keys: "f", action: "refit and regenerate", note: "fits the maze to the terminal now" },
    Binding { keys: "n", action: "new seed", note: "a fresh random seed, then generate" },
    Binding { keys: "b", action: "braid factor", note: "cycles 0.00, 0.25, 0.50" },
    Binding { keys: "?", action: "help", note: "the full keymap over the maze" },
    Binding { keys: "q  Esc", action: "quit", note: "restores the terminal, decision 18" },
];

pub fn draw_keymap(frame: &mut Frame, area: Rect) {
    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!("{:<10}", "key"), Style::new().fg(Color::DarkGray)),
            Span::styled(format!("{:<24}", "action"), Style::new().fg(Color::DarkGray)),
            Span::styled("note", Style::new().fg(Color::DarkGray)),
        ]),
        Line::from(""),
    ];
    for b in KEYMAP {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<10}", b.keys),
                Style::new().fg(Color::LightYellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{:<24}", b.action), Style::new()),
            Span::styled(b.note, Style::new().fg(Color::Gray)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(crate::render::speed_ladder_bar());
    lines.push(Line::from(Span::styled(
        "One ladder spans both regimes (decision 11). Single step ignores it.",
        Style::new().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "No in-app text entry in v1 (decision 19), so the seed changes only by n or by --seed.",
        Style::new().fg(Color::DarkGray),
    )));
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" candidate keymap ")),
        area,
    );
}

// ---------------------------------------------------------------- footprint

pub fn draw_footprint(frame: &mut Frame, area: Rect, term: (u16, u16)) {
    let sizes: [(u16, u16, &str); 3] = [
        (120, 30, "120 x 30"),
        (80, 24, "80 x 24"),
        (term.0, term.1, "this terminal"),
    ];

    let mut lines = vec![
        Line::from(Span::styled(
            "Largest maze that fits, in cells. Footprint is 4W+2 by 2H+1 (decision 15).",
            Style::new().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{:<26}", "layout"), Style::new().fg(Color::DarkGray)),
            Span::styled(format!("{:<12}", "chrome"), Style::new().fg(Color::DarkGray)),
            Span::styled(format!("{:<14}", "120 x 30"), Style::new().fg(Color::DarkGray)),
            Span::styled(format!("{:<14}", "80 x 24"), Style::new().fg(Color::DarkGray)),
            Span::styled(
                format!("{:<14}", format!("{} x {}", term.0, term.1)),
                Style::new().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
    ];

    for v in Variant::ALL {
        let (cc, cr) = v.chrome();
        let mut spans = vec![
            Span::styled(
                format!("{:<26}", format!("{}  {}", v.key(), v.name())),
                Style::new().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:<12}", format!("{cc}c {cr}r")),
                Style::new().fg(Color::Gray),
            ),
        ];
        for (cols, rows, _) in sizes {
            let cell = match v.fits(cols, rows) {
                Some((w, h)) => format!("{w} x {h}"),
                None => "-".to_string(),
            };
            spans.push(Span::styled(format!("{cell:<14}"), Style::new()));
        }
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "The mock maze is {} x {} cells, so it needs {} x {} screen cells plus chrome.",
            crate::mock::W,
            crate::mock::H,
            MAZE_COLS,
            MAZE_ROWS
        ),
        Style::new().fg(Color::Gray),
    )));
    lines.push(Line::from(""));
    for v in Variant::ALL {
        let (mc, mr) = v.min_size();
        lines.push(Line::from(Span::styled(
            format!("  {}  minimum terminal for the mock maze: {} x {}", v.key(), mc, mr),
            Style::new().fg(Color::DarkGray),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "An 80 x 24 terminal is the case that decides the default layout.",
        Style::new().fg(Color::LightYellow),
    )));

    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" footprint ")),
        area,
    );
}

// --------------------------------------------------------------- too small

pub fn draw_too_small_page(frame: &mut Frame, area: Rect) {
    let [note, panel] =
        Layout::vertical([Constraint::Length(4), Constraint::Fill(1)]).areas(area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                "Decision 9: a resize never changes the maze. Below the minimum the maze pane is",
                Style::new().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "replaced by this panel, not clipped. f refits and regenerates on purpose.",
                Style::new().fg(Color::Gray),
            )),
        ])
        .block(Block::bordered().title(" terminal too small ")),
        note,
    );

    let box_area = Rect::new(
        panel.x,
        panel.y,
        panel.width.min(64),
        panel.height.min(14),
    );
    frame.render_widget(
        Block::bordered()
            .title(Span::styled(" a 40 x 12 terminal ", Style::new().fg(Color::DarkGray)))
            .border_style(Style::new().fg(Color::DarkGray)),
        box_area,
    );
    let inner = Rect::new(box_area.x + 1, box_area.y + 1, 40.min(box_area.width.saturating_sub(2)), 12.min(box_area.height.saturating_sub(2)));
    crate::variants::draw_too_small(frame, inner, 84, 16);
}
