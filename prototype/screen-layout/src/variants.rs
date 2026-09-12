//! Three structurally different screen layouts. THROWAWAY.
//!
//! A: maze left, tall sidebar right.       Chrome takes columns.
//! B: full-width bands stacked vertically. Chrome takes rows.
//! C: maze maximal, two-row HUD.           Chrome takes almost nothing.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph};

use crate::mock;
use crate::render::{self, MAZE_COLS, MAZE_ROWS};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    A,
    B,
    C,
}

impl Variant {
    pub const ALL: [Variant; 3] = [Variant::A, Variant::B, Variant::C];

    pub fn key(self) -> &'static str {
        match self {
            Variant::A => "A",
            Variant::B => "B",
            Variant::C => "C",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Variant::A => "Sidebar",
            Variant::B => "Bands",
            Variant::C => "Maximal maze",
        }
    }

    /// Screen space this layout spends on everything that is not the maze
    /// pane's interior. The footprint page reads the same numbers, so the
    /// report cannot drift from the layout.
    pub fn chrome(self) -> (u16, u16) {
        match self {
            // sidebar + maze border | maze border + help row
            Variant::A => (SIDEBAR_W + 2, 2 + 1),
            // no side chrome | status + stats band + legend + help
            Variant::B => (0, 1 + STATS_BAND_H + 1 + 1),
            // no side chrome | hud + stats row
            Variant::C => (0, 2),
        }
    }

    /// Largest maze that fits a terminal of this size, in cells.
    pub fn fits(self, cols: u16, rows: u16) -> Option<(u16, u16)> {
        let (cc, cr) = self.chrome();
        let inner_cols = cols.checked_sub(cc)?;
        let inner_rows = rows.checked_sub(cr)?;
        let w = inner_cols.checked_sub(2)? / 4;
        let h = inner_rows.checked_sub(1)? / 2;
        if w == 0 || h == 0 { None } else { Some((w, h)) }
    }

    pub fn min_size(self) -> (u16, u16) {
        let (cc, cr) = self.chrome();
        (MAZE_COLS + cc, MAZE_ROWS + cr)
    }
}

const SIDEBAR_W: u16 = 34;
const STATS_BAND_H: u16 = 7;

pub fn draw(frame: &mut Frame, area: Rect, variant: Variant, ascii: bool, generating: bool) {
    let (min_cols, min_rows) = variant.min_size();
    if area.width < min_cols || area.height < min_rows {
        draw_too_small(frame, area, min_cols, min_rows);
        return;
    }
    match variant {
        Variant::A => draw_a(frame, area, ascii, generating),
        Variant::B => draw_b(frame, area, ascii, generating),
        Variant::C => draw_c(frame, area, ascii, generating),
    }
}

fn maze_paragraph(ascii: bool, generating: bool) -> Paragraph<'static> {
    Paragraph::new(render::maze_lines(ascii, generating)).alignment(Alignment::Center)
}

fn centred(area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(MAZE_COLS) / 2;
    let y = area.y + area.height.saturating_sub(MAZE_ROWS) / 2;
    Rect::new(x, y, MAZE_COLS.min(area.width), MAZE_ROWS.min(area.height))
}

// ------------------------------------------------------------------ variant A

fn draw_a(frame: &mut Frame, area: Rect, ascii: bool, generating: bool) {
    let [body, help] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
    let [left, right] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(SIDEBAR_W)]).areas(body);

    let maze_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(Line::from(vec![
            Span::raw(" maze "),
            Span::styled(
                format!("{}x{} ", mock::W, mock::H),
                Style::new().fg(Color::DarkGray),
            ),
        ]));
    let inner = maze_block.inner(left);
    frame.render_widget(maze_block, left);
    frame.render_widget(maze_paragraph(ascii, generating), centred(inner));

    let [status, stats, legend] = Layout::vertical([
        Constraint::Length(7),
        Constraint::Length(STATS_BAND_H + 2),
        Constraint::Fill(1),
    ])
    .areas(right);

    let status_lines = vec![
        kv("seed", mock::SEED.to_string()),
        kv("generator", mock::GENERATOR.to_string()),
        kv("solver", mock::CURRENT_RUN.algorithm.to_string()),
        Line::from(vec![
            Span::styled(format!("{:<11}", "phase"), Style::new().fg(Color::Gray)),
            Span::styled(render::phase_name(generating), render::phase_style(generating)),
        ]),
        render::speed_compact(),
    ];
    frame.render_widget(
        Paragraph::new(status_lines).block(Block::bordered().title(" run ")),
        status,
    );

    let mut stat_lines = if generating {
        render::gen_stats_rows()
    } else {
        render::stats_rows()
    };
    stat_lines.push(Line::from(""));
    if !generating {
        stat_lines.push(render::equal_path_note());
    }
    frame.render_widget(
        Paragraph::new(stat_lines).block(Block::bordered().title(" statistics ")),
        stats,
    );

    frame.render_widget(
        Paragraph::new(render::legend_lines(ascii, generating))
            .block(Block::bordered().title(" cell state ")),
        legend,
    );

    frame.render_widget(help_line(area.width), help);
}

fn kv(k: &'static str, v: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{k:<11}"), Style::new().fg(Color::Gray)),
        Span::styled(v, Style::new().add_modifier(Modifier::BOLD)),
    ])
}

// ------------------------------------------------------------------ variant B

fn draw_b(frame: &mut Frame, area: Rect, ascii: bool, generating: bool) {
    let [status, maze, stats, legend, help] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(STATS_BAND_H),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let status_spans = vec![
        Span::styled(" MazeLab ", Style::new().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        render::dim("seed "),
        render::val(mock::SEED.to_string()),
        render::dim("  gen "),
        render::val(mock::GENERATOR.to_string()),
        render::dim("  solver "),
        render::val(mock::CURRENT_RUN.algorithm.to_string()),
        render::dim("  phase "),
        Span::styled(render::phase_name(generating), render::phase_style(generating)),
        render::dim("  speed "),
        render::val(render::speed_label()),
    ];
    frame.render_widget(Paragraph::new(Line::from(status_spans)), status);

    frame.render_widget(maze_paragraph(ascii, generating), centred(maze));

    // Three columns: the run now, the run before it, and the maze it ran on.
    let [now, prev, maze_info] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .areas(stats);

    frame.render_widget(
        Paragraph::new(run_column(&mock::CURRENT_RUN, generating))
            .block(Block::bordered().title(Span::styled(
                " this run ",
                Style::new().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
            ))),
        now,
    );
    frame.render_widget(
        Paragraph::new(run_column(&mock::PREVIOUS_RUN, false))
            .block(Block::bordered().title(Span::styled(" previous run ", Style::new().fg(Color::DarkGray)))),
        prev,
    );
    frame.render_widget(
        Paragraph::new(vec![
            kv("maze", format!("{}x{}", mock::W, mock::H)),
            kv("braid", format!("{:.2}", mock::BRAID)),
            kv("carved", format!("{}", mock::W * mock::H - 1)),
            kv("start", "0,0".to_string()),
            render::equal_path_note(),
        ])
        .block(Block::bordered().title(" maze ")),
        maze_info,
    );

    frame.render_widget(
        Paragraph::new(render::legend_inline(ascii, generating, area.width < 100)),
        legend,
    );
    frame.render_widget(help_line(area.width), help);
}

fn run_column(run: &mock::RunStats, generating: bool) -> Vec<Line<'static>> {
    if generating {
        return vec![
            kv("algorithm", mock::GENERATOR.to_string()),
            kv("step", mock::GEN_STEPS.to_string()),
            kv("carved", format!("{} / {}", mock::GEN_CARVED_COUNT, mock::W * mock::H)),
            kv("frontier", mock::GEN_FRONTIER_COUNT.to_string()),
            Line::from(""),
        ];
    }
    vec![
        kv("algorithm", run.algorithm.to_string()),
        kv("step", run.steps.to_string()),
        kv("expanded", run.expanded.to_string()),
        kv("frontier", run.frontier.to_string()),
        kv("path len", run.path_len.to_string()),
    ]
}

// ------------------------------------------------------------------ variant C

fn draw_c(frame: &mut Frame, area: Rect, ascii: bool, generating: bool) {
    let [hud, maze, stats] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let hud_style = Style::new().fg(Color::Black).bg(Color::Gray);
    let hud_spans = vec![
        Span::styled(format!(" seed {} ", mock::SEED), hud_style.add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {} ", short_gen()), hud_style),
        Span::styled(format!(" {} ", mock::CURRENT_RUN.algorithm), hud_style),
        Span::styled(
            format!(" {} ", render::phase_name(generating)),
            Style::new()
                .fg(Color::Black)
                .bg(if generating { Color::LightMagenta } else { Color::LightGreen })
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {} ", render::speed_label()), hud_style),
        Span::styled(format!(" {}x{} ", mock::W, mock::H), hud_style),
        render::dim("  ? help   q quit"),
    ];
    frame.render_widget(Paragraph::new(Line::from(hud_spans)), hud);

    frame.render_widget(maze_paragraph(ascii, generating), centred(maze));

    let mut spans = if generating {
        vec![Span::styled(
            format!(
                "carving  step {}  carved {}/{}  frontier {}",
                mock::GEN_STEPS,
                mock::GEN_CARVED_COUNT,
                mock::W * mock::H,
                mock::GEN_FRONTIER_COUNT
            ),
            Style::new().fg(Color::LightMagenta),
        )]
    } else {
        render::inline_stats(&mock::CURRENT_RUN, "now")
    };
    if !generating {
        spans.push(render::dim("  |"));
        spans.extend(render::inline_stats(&mock::PREVIOUS_RUN, " prev"));
        spans.push(render::dim("  braid 0.00"));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), stats);
}

fn short_gen() -> &'static str {
    "RecBacktracker"
}

// ------------------------------------------------------------------- shared

fn help_line(width: u16) -> Paragraph<'static> {
    let short: &[(&str, &str)] = &[
        ("g", "generate"),
        ("s", "solve"),
        ("Space", "pause"),
        (".", "step"),
        ("+/-", "speed"),
        ("?", "help"),
        ("q", "quit"),
    ];
    let full: &[(&str, &str)] = &[
        ("g", "generate"),
        ("G", "instant"),
        ("s", "solve"),
        ("Space", "pause"),
        (".", "step"),
        ("+/-", "speed"),
        ("1-2", "gen"),
        ("3-5", "solver"),
        ("n", "new seed"),
        ("f", "refit"),
        ("?", "help"),
        ("q", "quit"),
    ];
    let items = if width < 100 { short } else { full };
    let mut spans = Vec::new();
    for (k, label) in items {
        spans.push(render::key(k));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*label, Style::new().fg(Color::Gray)));
        spans.push(Span::raw("  "));
    }
    Paragraph::new(Line::from(spans))
}

pub fn draw_too_small(frame: &mut Frame, area: Rect, need_cols: u16, need_rows: u16) {
    frame.render_widget(Clear, area);
    let w = 46.min(area.width);
    let h = 8.min(area.height);
    let rect = Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    );
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Terminal too small",
            Style::new().fg(Color::LightRed).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(format!("need {need_cols} x {need_rows}")).alignment(Alignment::Center),
        Line::from(format!("have {} x {}", area.width, area.height)).alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(
            "resize, or press f to refit and regenerate",
            Style::new().fg(Color::Gray),
        ))
        .alignment(Alignment::Center),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(Color::LightRed)),
        ),
        rect,
    );
}
