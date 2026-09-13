//! Item 8 of tier 3 of the test plan: the two too-small panels of section 13,
//! and the order in which a panel degrades.
//!
//! **There are two thresholds, not one.** The maze-fit panel replaces the maze
//! pane only, and the rest of the chrome keeps rendering. The layout-floor
//! panel replaces the whole screen. The two have different wordings.

use mazelab::app::{Action, App, Phase, Startup, capacity};
use mazelab::maze::Maze;
use mazelab::ui::{self, panel};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

/// The startup options every test below starts from: a fixed seed, the
/// Recursive Backtracker and A\*, which are the defaults of section 9.
fn startup() -> Startup {
    Startup {
        seed: 7,
        width: None,
        height: None,
        ascii: false,
        generator_ix: 0,
        solver_ix: 2,
    }
}

/// A terminal of `cols` x `rows` with one frame of `app` drawn on it.
fn terminal(app: &App, cols: u16, rows: u16) -> Terminal<TestBackend> {
    // `TestBackend::Error` is `Infallible`, so the two patterns are
    // irrefutable and no test helper here returns a `Result`.
    let Ok(mut terminal) = Terminal::new(TestBackend::new(cols, rows));
    let Ok(_) = terminal.draw(|frame| ui::render(frame, app));
    terminal
}

/// Draws one frame of `app` on a terminal of `cols` x `rows`.
fn draw(app: &App, cols: u16, rows: u16) -> Buffer {
    terminal(app, cols, rows).backend().buffer().clone()
}

/// The application at startup on a terminal of `cols` x `rows`, drawn there.
fn draw_on(cols: u16, rows: u16) -> Buffer {
    draw(&App::new(startup(), capacity(cols, rows)), cols, rows)
}

/// The symbols of one screen row, joined.
fn row(buffer: &Buffer, y: u16) -> String {
    (0..buffer.area.width)
        .map(|x| buffer[(x, y)].symbol())
        .collect()
}

/// Every row of the buffer, joined.
fn screen_rows(buffer: &Buffer) -> Vec<String> {
    (0..buffer.area.height).map(|y| row(buffer, y)).collect()
}

/// The layout-floor panel of section 13.5, at a terminal of 60 x 18.
const FLOOR_60_BY_18: [&str; 8] = [
    "╭────────────────────────────────────────────╮",
    "│             Terminal too small             │",
    "│                                            │",
    "│                need 79 x 19                │",
    "│                have 60 x 18                │",
    "│                                            │",
    "│             resize to continue             │",
    "╰────────────────────────────────────────────╯",
];

// ---------------------------------------------------------------------------
// The layout floor
// ---------------------------------------------------------------------------

#[test]
fn below_the_layout_floor_the_whole_screen_is_the_panel() {
    // The box is 46 x 8, centred in 60 x 18: 7 columns on the left and 5 rows
    // on top. Nothing else is drawn: no status row, no statistics band, no
    // legend and no help row.
    let app = App::new(startup(), capacity(60, 18));
    let red = Style::new().fg(Color::LightRed);
    let mut expected = vec![Line::from(""); 18];
    for (y, text) in (5..).zip(FLOOR_60_BY_18) {
        expected[y] = Line::from(vec![
            Span::raw(" ".repeat(7)),
            Span::styled(text, red),
            Span::raw(" ".repeat(7)),
        ]);
    }
    terminal(&app, 60, 18)
        .backend()
        .assert_buffer_lines(expected);
}

// ---------------------------------------------------------------------------
// The maze fit
// ---------------------------------------------------------------------------

/// The maze-fit panel of section 13.5, for a 29 x 9 maze on a terminal of
/// 100 x 24.
const MAZE_FIT_29_BY_9: [&str; 8] = [
    "╭────────────────────────────────────────────╮",
    "│      Maze too large for this terminal      │",
    "│                                            │",
    "│         maze 29 x 9 needs 118 x 29         │",
    "│            terminal is 100 x 24            │",
    "│                                            │",
    "│ resize, or press f to refit and regenerate │",
    "╰────────────────────────────────────────────╯",
];

#[test]
fn an_oversized_maze_gives_the_maze_fit_panel_in_the_pane_only() {
    // Section 13.2: a stated size is not clamped to capacity, which is 24 x 6
    // at 100 x 24. The terminal is above the layout floor, so the message is
    // that the maze is too large, not that the terminal is too small.
    let oversized = Startup {
        width: Some(29),
        height: Some(9),
        ..startup()
    };
    let app = App::new(oversized, capacity(100, 24));
    let buffer = draw(&app, 100, 24);
    let screen = screen_rows(&buffer);

    // The pane is rows 1 to 14, 100 x 14, and the box is 46 x 8 in it: 27
    // columns on the left and 3 rows on top.
    assert!(
        screen[0].starts_with("MazeLab  seed 7  "),
        "{:?}",
        screen[0]
    );
    for (y, text) in screen.iter().enumerate().take(15).skip(1) {
        let expected = match y {
            4..=11 => format!("{:27}{}{:27}", "", MAZE_FIT_29_BY_9[y - 4], ""),
            _ => " ".repeat(100),
        };
        assert_eq!(text, &expected, "pane row {y}");
    }
    assert_eq!(buffer[(27, 4)].fg, Color::LightRed, "the corner of the box");
    assert_eq!(buffer[(33, 5)].fg, Color::LightRed, "the title");

    // The status row, the statistics band, the legend and the help row keep
    // rendering.
    assert!(screen[15].contains("this run"), "{:?}", screen[15]);
    assert!(screen[16].trim_end().ends_with("29x9│"), "{:?}", screen[16]);
    assert!(screen[22].contains("frontier"), "{:?}", screen[22]);
    assert!(screen[23].contains("q quit"), "{:?}", screen[23]);
    assert!(
        screen
            .iter()
            .all(|text| !text.contains("Terminal too small"))
    );
}

// ---------------------------------------------------------------------------
// The sizes of section 7.1, and one size below each threshold
// ---------------------------------------------------------------------------

/// The title of the maze-fit panel.
const MAZE_FIT: &str = "Maze too large for this terminal";

/// The title of the layout-floor panel.
const FLOOR: &str = "Terminal too small";

/// How many screen rows hold `text`.
fn rows_with(buffer: &Buffer, text: &str) -> usize {
    screen_rows(buffer)
        .iter()
        .filter(|row| row.contains(text))
        .count()
}

#[test]
fn the_sizes_of_section_7_1_draw_the_maze_and_no_panel() {
    for (cols, rows) in [(79, 19), (80, 24), (120, 30)] {
        let buffer = draw_on(cols, rows);
        assert_eq!(rows_with(&buffer, MAZE_FIT), 0, "{cols} x {rows}");
        assert_eq!(rows_with(&buffer, FLOOR), 0, "{cols} x {rows}");
        assert!(row(&buffer, 0).starts_with("MazeLab"), "{cols} x {rows}");
        assert!(row(&buffer, 1).contains("████"), "{cols} x {rows}");
    }
}

#[test]
fn one_row_below_the_maze_fit_gives_the_maze_fit_panel() {
    // The maze of an 80 x 24 terminal is 19 x 6 and needs 78 x 23. At 80 x 22
    // the terminal is above the layout floor, so only the maze pane goes.
    let mut app = App::new(startup(), capacity(80, 24));
    assert!(app.set_capacity(capacity(80, 22)));
    let buffer = draw(&app, 80, 22);
    assert_eq!(rows_with(&buffer, MAZE_FIT), 1);
    assert_eq!(rows_with(&buffer, "maze 19 x 6 needs 78 x 23"), 1);
    assert_eq!(rows_with(&buffer, "terminal is 80 x 22"), 1);
    assert_eq!(rows_with(&buffer, FLOOR), 0);
    assert!(row(&buffer, 0).starts_with("MazeLab"));
    assert!(row(&buffer, 21).contains("q quit"));
}

#[test]
fn one_column_or_one_row_below_the_layout_floor_gives_the_floor_panel() {
    for (cols, rows) in [(78, 19), (79, 18)] {
        let mut app = App::new(startup(), capacity(79, 19));
        assert!(app.set_capacity(capacity(cols, rows)));
        let buffer = draw(&app, cols, rows);
        assert_eq!(rows_with(&buffer, FLOOR), 1, "{cols} x {rows}");
        assert_eq!(rows_with(&buffer, "need 79 x 19"), 1, "{cols} x {rows}");
        let have = format!("have {cols} x {rows}");
        assert_eq!(rows_with(&buffer, &have), 1, "{cols} x {rows}");
        assert_eq!(rows_with(&buffer, MAZE_FIT), 0, "{cols} x {rows}");
        assert_eq!(rows_with(&buffer, "MazeLab"), 0, "no status row");
        assert_eq!(rows_with(&buffer, "frontier"), 0, "no legend");
        assert_eq!(rows_with(&buffer, "quit"), 0, "no help row");
    }
}

#[test]
fn below_the_layout_floor_an_oversized_maze_gets_the_floor_panel_with_no_f() {
    // Below the floor `f` is inert, so the panel names the one remedy that
    // works, whatever the size of the maze. Section 13.3.
    let oversized = Startup {
        width: Some(60),
        ..startup()
    };
    let app = App::new(oversized, capacity(60, 18));
    let buffer = draw(&app, 60, 18);
    assert_eq!(rows_with(&buffer, FLOOR), 1);
    assert_eq!(rows_with(&buffer, "resize to continue"), 1);
    assert_eq!(rows_with(&buffer, "press f"), 0);
    assert_eq!(rows_with(&buffer, MAZE_FIT), 0);
}

#[test]
fn the_help_overlay_is_not_drawn_below_the_layout_floor() {
    let mut app = App::new(startup(), capacity(60, 18));
    assert!(app.apply(Action::ToggleHelp));
    let buffer = draw(&app, 60, 18);
    assert_eq!(rows_with(&buffer, "keymap"), 0);
    assert_eq!(rows_with(&buffer, FLOOR), 1);
}

#[test]
fn a_run_behind_the_maze_fit_panel_is_paused_and_stays_paused() {
    // Section 13.4: entering too small auto-pauses the run, and growing the
    // terminal back shows the maze again, still paused.
    let mut app = App::new(startup(), capacity(80, 24));
    assert!(app.apply(Action::Solve));
    assert!(app.set_capacity(capacity(80, 22)));
    assert!(matches!(app.phase, Phase::Paused(_)));
    assert_eq!(rows_with(&draw(&app, 80, 22), MAZE_FIT), 1);

    assert!(app.set_capacity(capacity(80, 24)));
    let buffer = draw(&app, 80, 24);
    assert_eq!(rows_with(&buffer, MAZE_FIT), 0);
    assert!(row(&buffer, 0).contains("phase Paused"));
}

// ---------------------------------------------------------------------------
// The degradation order of section 13.5
// ---------------------------------------------------------------------------

/// The layout-floor panel on a terminal of `cols` x `rows`, row by row.
fn floor_on(cols: u16, rows: u16) -> Vec<String> {
    screen_rows(&draw_on(cols, rows))
}

#[test]
fn the_floor_panel_is_boxed_where_the_box_fits() {
    assert_eq!(
        floor_on(46, 8),
        [
            "╭────────────────────────────────────────────╮",
            "│             Terminal too small             │",
            "│                                            │",
            "│                need 79 x 19                │",
            "│                have 46 x 8                 │",
            "│                                            │",
            "│             resize to continue             │",
            "╰────────────────────────────────────────────╯",
        ]
    );
}

#[test]
fn the_floor_panel_drops_the_border_first() {
    // One column short of the box. The six rows stay, centred in 45 x 8.
    assert_eq!(
        floor_on(45, 8),
        [
            "                                             ",
            "             Terminal too small              ",
            "                                             ",
            "                need 79 x 19                 ",
            "                have 45 x 8                  ",
            "                                             ",
            "             resize to continue              ",
            "                                             ",
        ]
    );
    // Six rows is the height the six rows need.
    assert_eq!(
        floor_on(45, 6)[5],
        "             resize to continue              "
    );
}

#[test]
fn the_floor_panel_then_drops_the_hint_then_have_then_need() {
    // Five rows cannot hold the hint and the row above it.
    assert_eq!(
        floor_on(45, 5),
        [
            "             Terminal too small              ",
            "                                             ",
            "                need 79 x 19                 ",
            "                have 45 x 5                  ",
            "                                             ",
        ]
    );
    assert_eq!(
        floor_on(45, 3),
        [
            "             Terminal too small              ",
            "                                             ",
            "                need 79 x 19                 ",
        ]
    );
    assert_eq!(
        floor_on(45, 2),
        [
            "             Terminal too small              ",
            "                                             ",
        ]
    );
}

#[test]
fn the_first_line_survives_to_the_smallest_terminal_that_can_hold_it() {
    assert_eq!(floor_on(18, 1), ["Terminal too small"]);
    // It is never clipped in the middle, so one column fewer draws nothing.
    assert_eq!(floor_on(17, 1), [" ".repeat(17)]);
    assert_eq!(floor_on(17, 8), vec![" ".repeat(17); 8]);
}

/// The maze-fit panel of a 29 x 9 maze on a terminal of 100 x 24, drawn in an
/// area of `cols` x `rows`, row by row.
///
/// On the screen the maze pane is never narrower than 79 x 9, so the box
/// always fits there. The panel is drawn on its own to show its degradation.
fn maze_fit_in(cols: u16, rows: u16) -> Vec<String> {
    let maze = Maze::new(29, 9);
    let maze_fit = panel::maze_fit(&maze, Rect::new(0, 0, 100, 24));
    let area = Rect::new(0, 0, cols, rows);
    let mut buffer = Buffer::empty(area);
    (&maze_fit).render(area, &mut buffer);
    screen_rows(&buffer)
}

#[test]
fn the_maze_fit_panel_drops_a_line_that_is_too_wide_and_every_line_below_it() {
    // In 44 columns the 42-column hint still fits without the border.
    assert_eq!(
        maze_fit_in(44, 8),
        [
            "                                            ",
            "      Maze too large for this terminal      ",
            "                                            ",
            "         maze 29 x 9 needs 118 x 29         ",
            "            terminal is 100 x 24            ",
            "                                            ",
            " resize, or press f to refit and regenerate ",
            "                                            ",
        ]
    );
    // In 41 columns it does not, and it goes whole. The rest stays.
    assert_eq!(
        maze_fit_in(41, 8),
        [
            "                                         ",
            "                                         ",
            "    Maze too large for this terminal     ",
            "                                         ",
            "       maze 29 x 9 needs 118 x 29        ",
            "          terminal is 100 x 24           ",
            "                                         ",
            "                                         ",
        ]
    );
    // The title is the last line to go, so in 31 columns nothing is left.
    assert_eq!(maze_fit_in(31, 8), vec![" ".repeat(31); 8]);
}
