//! Shared drawing helpers. THROWAWAY.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::mock::{self, RunStats};
use crate::theme::{self, gen_look, solver_look};

/// Screen footprint of the display grid: 4W+2 columns by 2H+1 rows
/// (decision 15: two screen columns per display-grid position).
pub const MAZE_COLS: u16 = (4 * mock::W + 2) as u16;
pub const MAZE_ROWS: u16 = (2 * mock::H + 1) as u16;

pub fn maze_lines(ascii: bool, generating: bool) -> Vec<Line<'static>> {
    let gh = 2 * mock::H + 1;
    let gw = 2 * mock::W + 1;
    (0..gh)
        .map(|y| {
            let spans: Vec<Span> = (0..gw)
                .map(|x| {
                    let look = if generating {
                        gen_look(mock::gen_at(x, y), ascii)
                    } else {
                        solver_look(mock::solver_at(x, y), ascii)
                    };
                    Span::styled(look.glyph, look.style())
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}

pub fn dim(s: &str) -> Span<'static> {
    Span::styled(s.to_string(), Style::new().fg(Color::DarkGray))
}

pub fn key(s: &str) -> Span<'static> {
    Span::styled(s.to_string(), Style::new().fg(Color::LightYellow).add_modifier(Modifier::BOLD))
}

pub fn val(s: String) -> Span<'static> {
    Span::styled(s, Style::new().add_modifier(Modifier::BOLD))
}

pub fn speed_label() -> String {
    format!("{}x", mock::SPEED_LADDER[mock::SPEED_RUNG])
}

/// The compact readout, for a layout with no room for the ladder.
pub fn speed_compact() -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{:<11}", "speed"), Style::new().fg(Color::Gray)),
        Span::styled(speed_label(), Style::new().fg(Color::LightYellow).add_modifier(Modifier::BOLD)),
        dim(&format!("  rung {} of {}", mock::SPEED_RUNG + 1, mock::SPEED_LADDER.len())),
    ])
}

pub fn speed_ladder_bar() -> Line<'static> {
    let mut spans = vec![dim("speed ")];
    for (i, rung) in mock::SPEED_LADDER.iter().enumerate() {
        let style = if i == mock::SPEED_RUNG {
            Style::new().fg(Color::Black).bg(Color::LightYellow).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::DarkGray)
        };
        spans.push(Span::styled(format!(" {rung} "), style));
    }
    spans.push(dim(" steps/frame"));
    Line::from(spans)
}

/// The solver statistics of decision 8, current run beside the previous one.
pub fn stats_rows() -> Vec<Line<'static>> {
    let c = &mock::CURRENT_RUN;
    let p = &mock::PREVIOUS_RUN;
    let row = |label: &'static str, a: String, b: String| {
        Line::from(vec![
            Span::styled(format!("{label:<10}"), Style::new().fg(Color::Gray)),
            Span::styled(format!("{a:>7}"), Style::new().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{b:>9}"), Style::new().fg(Color::DarkGray)),
        ])
    };
    vec![
        Line::from(vec![
            Span::styled(format!("{:<10}", ""), Style::new()),
            Span::styled(format!("{:>7}", c.algorithm), Style::new().fg(Color::LightCyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:>9}", p.algorithm), Style::new().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<10}", ""), Style::new()),
            Span::styled(format!("{:>7}", "now"), Style::new().fg(Color::DarkGray)),
            Span::styled(format!("{:>9}", "previous"), Style::new().fg(Color::DarkGray)),
        ]),
        row("step", c.steps.to_string(), p.steps.to_string()),
        row("expanded", c.expanded.to_string(), p.expanded.to_string()),
        row("frontier", c.frontier.to_string(), p.frontier.to_string()),
        row("path len", c.path_len.to_string(), p.path_len.to_string()),
    ]
}

/// Decision 6: with braiding off, every solver returns the same path, so the
/// panel has to say that an equal path length is the expected result.
pub fn equal_path_note() -> Line<'static> {
    Line::from(vec![dim("equal path expected")])
}

pub fn gen_stats_rows() -> Vec<Line<'static>> {
    let row = |label: &'static str, a: String| {
        Line::from(vec![
            Span::styled(format!("{label:<10}"), Style::new().fg(Color::Gray)),
            Span::styled(format!("{a:>7}"), Style::new().add_modifier(Modifier::BOLD)),
        ])
    };
    vec![
        Line::from(vec![Span::styled(
            format!("{:<10}{:>7}", "", "carving"),
            Style::new().fg(Color::LightMagenta).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        row("step", mock::GEN_STEPS.to_string()),
        row("carved", format!("{} / {}", mock::GEN_CARVED_COUNT, mock::W * mock::H)),
        row("frontier", mock::GEN_FRONTIER_COUNT.to_string()),
        Line::from(""),
    ]
}

pub fn legend_lines(ascii: bool, generating: bool) -> Vec<Line<'static>> {
    if generating {
        theme::GEN_STATES
            .iter()
            .map(|(state, name)| legend_row(gen_look(*state, ascii), name))
            .collect()
    } else {
        theme::SOLVER_STATES
            .iter()
            .map(|(state, name)| legend_row(solver_look(*state, ascii), name))
            .collect()
    }
}

fn legend_row(look: crate::theme::Look, name: &'static str) -> Line<'static> {
    let swatch = Span::styled(look.glyph, look.style());
    Line::from(vec![swatch, Span::raw(" "), Span::raw(name)])
}

/// The legend on one row, for the layouts that have no sidebar. Below about
/// 100 columns the full labels do not fit, so the row switches to short ones.
pub fn legend_inline(ascii: bool, generating: bool, compact: bool) -> Line<'static> {
    let mut spans = Vec::new();
    let push = |spans: &mut Vec<Span<'static>>, look: crate::theme::Look, name: &'static str| {
        spans.push(Span::styled(look.glyph, look.style()));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(name, Style::new().fg(Color::Gray)));
        spans.push(Span::raw("  "));
    };
    if generating {
        for (i, (s, n)) in theme::GEN_STATES.iter().enumerate() {
            let label = if compact { theme::GEN_SHORT[i] } else { n };
            push(&mut spans, gen_look(*s, ascii), label);
        }
    } else {
        for (i, (s, n)) in theme::SOLVER_STATES.iter().enumerate() {
            let label = if compact { theme::SOLVER_SHORT[i] } else { n };
            push(&mut spans, solver_look(*s, ascii), label);
        }
    }
    Line::from(spans)
}

pub fn phase_name(generating: bool) -> &'static str {
    if generating { "Generating" } else { "Solved" }
}

pub fn phase_style(generating: bool) -> Style {
    let c = if generating { Color::LightMagenta } else { Color::LightGreen };
    Style::new().fg(c).add_modifier(Modifier::BOLD)
}

pub fn inline_stats(run: &RunStats, prefix: &'static str) -> Vec<Span<'static>> {
    vec![
        Span::styled(prefix, Style::new().fg(Color::DarkGray)),
        Span::styled(
            format!(
                " {} step {} exp {} front {} path {}",
                run.algorithm, run.steps, run.expanded, run.frontier, run.path_len
            ),
            Style::new().fg(Color::Gray),
        ),
    ]
}
