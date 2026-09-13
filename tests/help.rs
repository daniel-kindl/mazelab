//! Item 7 of tier 3 of the test plan: each form of the help row of section
//! 7.7 fits its breakpoint.
//!
//! **The help row never clips.** The short form fits the 79-column layout
//! floor, and the full form fits the 100-column breakpoint.

use mazelab::app::{App, FLOOR_COLS, Startup, capacity};
use mazelab::ui::{self, help, layout};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Color;

/// The text of a line, with no style.
fn text(line: &ratatui::text::Line) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

#[test]
fn below_100_columns_the_help_row_names_seven_entries_in_67_columns() {
    let row = help::row(99);
    assert_eq!(
        text(&row),
        "g generate  s solve  Space pause  . step  +/- speed  ? help  q quit"
    );
    assert_eq!(row.width(), 67);
}

#[test]
fn at_100_columns_the_help_row_adds_instant_and_the_selection_in_99_columns() {
    let row = help::row(100);
    assert_eq!(
        text(&row),
        "g generate  G instant  s solve  Space pause  . step  +/- speed  1-2 gen  3-5 solver  ? help  q quit"
    );
    assert_eq!(row.width(), 99);
}

#[test]
fn each_form_fits_its_breakpoint() {
    assert!(help::row(layout::BREAKPOINT).width() <= usize::from(layout::BREAKPOINT));
    assert!(help::row(layout::BREAKPOINT - 1).width() <= usize::from(FLOOR_COLS));
}

#[test]
fn the_screen_draws_the_whole_help_row_with_the_keys_in_yellow() {
    let cases = [
        (
            100,
            "g generate  G instant  s solve  Space pause  . step  +/- speed  1-2 gen  3-5 solver  ? help  q quit",
        ),
        (
            79,
            "g generate  s solve  Space pause  . step  +/- speed  ? help  q quit",
        ),
    ];
    for (cols, expected) in cases {
        let startup = Startup {
            seed: 7,
            width: None,
            height: None,
            ascii: false,
            generator_ix: 0,
            solver_ix: 2,
        };
        let app = App::new(startup, capacity(cols, 24));
        let Ok(mut terminal) = Terminal::new(TestBackend::new(cols, 24));
        let Ok(_) = terminal.draw(|frame| ui::render(frame, &app));
        let buffer = terminal.backend().buffer();
        let drawn: String = (0..cols).map(|x| buffer[(x, 23)].symbol()).collect();
        assert_eq!(drawn.trim_end(), expected, "the help row at {cols} columns");
        // `Space` starts after `g generate  s solve  `, 21 columns in.
        let space = if cols >= 100 { 32 } else { 21 };
        for x in [0, space, space + 4] {
            assert_eq!(buffer[(x, 23)].fg, Color::Yellow, "column {x} is a key");
        }
        assert_eq!(buffer[(2, 23)].fg, Color::Reset, "a label is not a key");
    }
}
