//! The screen of section 7: the bands, the maze pane, the status row, the
//! statistics band of section 8 and the help overlay. Item 9 of tier 3 of the
//! test plan is here: a full-screen snapshot of a small solved maze.
//!
//! Every test draws into a `TestBackend`, which renders into memory and needs
//! no terminal. No test can render a glyph on a real font; these tests hold
//! where each glyph goes and which glyph it is.

use mazelab::StepOutcome;
use mazelab::app::{Action, App, Startup, capacity};
use mazelab::ui::{self, layout};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

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

/// The application at startup on a terminal of `cols` x `rows`.
fn app_on(cols: u16, rows: u16) -> App {
    App::new(startup(), capacity(cols, rows))
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

/// The symbols of one screen row, joined.
fn row(buffer: &Buffer, y: u16) -> String {
    (0..buffer.area.width)
        .map(|x| buffer[(x, y)].symbol())
        .collect()
}

/// The symbols of one screen column between two rows, inclusive, joined.
fn column(buffer: &Buffer, x: u16, rows: std::ops::RangeInclusive<u16>) -> String {
    rows.map(|y| buffer[(x, y)].symbol()).collect()
}

/// Holds the outer border of the maze to the rectangle that starts at
/// `(left, top)` and measures `width` x `height` screen cells, and holds the
/// cells around that rectangle blank.
///
/// The outer border of the display grid is always a wall, so a maze of the
/// right size in the right place draws exactly this frame of `█`.
fn assert_maze_at(buffer: &Buffer, left: u16, top: u16, width: u16, height: u16) {
    let right = left + width - 1;
    let bottom = top + height - 1;
    let wall_row = "█".repeat(usize::from(width));
    let wall_column = "█".repeat(usize::from(height));
    let top_row = row(buffer, top);
    let bottom_row = row(buffer, bottom);
    let slice = |text: &str| -> String {
        text.chars()
            .skip(usize::from(left))
            .take(usize::from(width))
            .collect()
    };
    assert_eq!(slice(&top_row), wall_row, "the top wall, on row {top}");
    assert_eq!(
        slice(&bottom_row),
        wall_row,
        "the bottom wall, on row {bottom}"
    );
    assert_eq!(
        column(buffer, left, top..=bottom),
        wall_column,
        "the left wall"
    );
    assert_eq!(
        column(buffer, right, top..=bottom),
        wall_column,
        "the right wall"
    );
    let blank = " ".repeat(usize::from(height));
    assert_eq!(
        column(buffer, left - 1, top..=bottom),
        blank,
        "left of the maze"
    );
    assert_eq!(
        column(buffer, right + 1, top..=bottom),
        blank,
        "right of the maze"
    );
}

// ---------------------------------------------------------------------------
// The layout of section 7.1
// ---------------------------------------------------------------------------

#[test]
fn a_120_by_30_terminal_shows_a_29_by_9_maze_centred_in_the_pane() {
    let app = app_on(120, 30);
    assert_eq!((app.maze.width(), app.maze.height()), (29, 9));
    let buffer = draw(&app, 120, 30);
    // The pane is rows 1 to 20, which is 20 rows, and the maze is 19 rows, so
    // the one row of slack falls below it. The maze is 118 columns of 120.
    assert_maze_at(&buffer, 1, 1, 118, 19);
}

#[test]
fn an_80_by_24_terminal_shows_a_19_by_6_maze_centred_in_the_pane() {
    let app = app_on(80, 24);
    assert_eq!((app.maze.width(), app.maze.height()), (19, 6));
    let buffer = draw(&app, 80, 24);
    // The pane is rows 1 to 14 and the maze is 13 rows. 78 columns of 80.
    assert_maze_at(&buffer, 1, 1, 78, 13);
}

#[test]
fn a_pane_with_slack_centres_the_maze_on_both_axes() {
    // At 90 x 26 capacity is 22 x 7. A --width of 20 and a --height of 6
    // give a maze of 82 x 13 in a pane of 90 x 16, so the pane has 8 columns
    // and 3 rows of slack. 4 columns go on the left and 1 row goes on top.
    let startup = Startup {
        width: Some(20),
        height: Some(6),
        ..startup()
    };
    let app = App::new(startup, capacity(90, 26));
    let buffer = draw(&app, 90, 26);
    assert_maze_at(&buffer, 4, 2, 82, 13);
}

#[test]
fn the_chrome_costs_no_columns_and_ten_rows() {
    for (cols, rows) in [(79, 19), (80, 24), (120, 30), (300, 100)] {
        let bands = layout::bands(Rect::new(0, 0, cols, rows));
        assert_eq!(
            bands.maze,
            Rect::new(0, 1, cols, rows - 10),
            "{cols} x {rows}"
        );
        assert_eq!(bands.status, Rect::new(0, 0, cols, 1));
        assert_eq!(bands.stats, Rect::new(0, rows - 9, cols, 7));
        assert_eq!(bands.legend, Rect::new(0, rows - 2, cols, 1));
        assert_eq!(bands.help, Rect::new(0, rows - 1, cols, 1));
    }
}

// ---------------------------------------------------------------------------
// The maze pane
// ---------------------------------------------------------------------------

/// The number of times `glyph` is drawn above the statistics band: on the
/// status row and in the maze pane.
///
/// The legend row draws one swatch of every cell state, so a count over the
/// whole screen would count the legend too.
fn count(buffer: &Buffer, glyph: &str) -> usize {
    (0..buffer.area.height - layout::STATS_ROWS - 2)
        .map(|y| row(buffer, y).matches(glyph).count())
        .sum()
}

#[test]
fn a_finished_maze_shows_the_start_and_the_goal_and_no_current_cell() {
    // Randomized Prim keeps answering the last cell it carved after its run is
    // done. `Ready` has no run going on, so that cell must not draw as `@@`.
    // The start and the goal of the next run are drawn at their corners.
    let startup = Startup {
        generator_ix: 1,
        ..startup()
    };
    let app = App::new(startup, capacity(80, 24));
    let buffer = draw(&app, 80, 24);
    assert_eq!(count(&buffer, "@@"), 0);
    assert_eq!(count(&buffer, "SS"), 1);
    assert_eq!(count(&buffer, "GG"), 1);
    assert!(row(&buffer, 2).starts_with(" ██SS"));
    assert!(row(&buffer, 12).ends_with("GG██ "));
}

#[test]
fn the_ascii_glyph_set_draws_the_whole_maze_and_no_block_glyph() {
    let startup = Startup {
        ascii: true,
        ..startup()
    };
    let mut app = App::new(startup, capacity(80, 24));
    assert!(app.apply(Action::Solve));
    for _ in 0..20 {
        assert_eq!(app.step(), StepOutcome::Stepped);
    }
    let buffer = draw(&app, 80, 24);
    for block in ["█", "▓", "▒", "░"] {
        assert_eq!(count(&buffer, block), 0, "{block} is not in the ASCII set");
    }
    assert_eq!(&row(&buffer, 1)[1..79], "#".repeat(78));
    for glyph in ["SS", "GG", "@@", "::", ".."] {
        assert!(count(&buffer, glyph) > 0, "a part-way run draws {glyph}");
    }
}

#[test]
fn a_generation_run_draws_its_current_cell_and_its_frontier() {
    // Part-way through, the Recursive Backtracker holds a stack. Its top is
    // the current cell and the rest draws as frontier. A generation run
    // draws with the generation glyphs and colours only.
    let mut app = app_on(80, 24);
    assert!(app.apply(Action::Generate));
    assert!(app.apply(Action::PauseResume));
    for _ in 0..10 {
        assert!(app.apply(Action::SingleStep));
    }
    let buffer = draw(&app, 80, 24);
    assert_eq!(count(&buffer, "@@"), 1);
    assert!(count(&buffer, "▒▒") > 0, "the stack draws as frontier");
    for solver_glyph in ["░░", "▓▓", "SS", "GG"] {
        assert_eq!(
            count(&buffer, solver_glyph),
            0,
            "{solver_glyph} is not a generation glyph"
        );
    }
    let frontier = (0..24)
        .flat_map(|y| (0..80).map(move |x| (x, y)))
        .find(|&(x, y)| buffer[(x, y)].symbol() == "▒")
        .map(|(x, y)| buffer[(x, y)].fg);
    assert_eq!(frontier, Some(Color::LightMagenta));
}

// ---------------------------------------------------------------------------
// The status row of section 7.2
// ---------------------------------------------------------------------------

#[test]
fn the_status_row_names_the_seed_the_full_names_the_phase_and_the_speed() {
    let mut app = app_on(120, 30);
    assert!(app.apply(Action::ChooseGenerator(1)));
    assert!(app.apply(Action::ChooseSolver(1)));
    assert!(app.apply(Action::Slower));
    let buffer = draw(&app, 120, 30);
    assert_eq!(
        row(&buffer, 0).trim_end(),
        "MazeLab  seed 7  gen Randomized Prim  solver BFS  phase Ready  speed 4x"
    );

    while app.apply(Action::Slower) {}
    assert!(app.apply(Action::Solve));
    assert!(app.apply(Action::PauseResume));
    let buffer = draw(&app, 120, 30);
    assert_eq!(
        row(&buffer, 0).trim_end(),
        "MazeLab  seed 7  gen Randomized Prim  solver BFS  phase Paused  speed 0.5x"
    );
}

// ---------------------------------------------------------------------------
// The statistics band of section 8.1
// ---------------------------------------------------------------------------

/// The content of the three columns of the statistics band on screen row `y`,
/// each with its runs of spaces collapsed to one and trimmed.
fn band_row(buffer: &Buffer, y: u16) -> [String; 3] {
    let text = row(buffer, y);
    let cells: Vec<&str> = text.split('│').collect();
    assert_eq!(
        cells.len(),
        7,
        "three bordered columns on row {y}: {text:?}"
    );
    [cells[1], cells[3], cells[5]].map(|cell| cell.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// The five content rows of the statistics band, column by column.
fn band(buffer: &Buffer) -> [[String; 5]; 3] {
    let top = buffer.area.height - 9;
    let rows: Vec<[String; 3]> = (top + 1..=top + 5).map(|y| band_row(buffer, y)).collect();
    [0, 1, 2].map(|c| [0, 1, 2, 3, 4].map(|r| rows[r][c].clone()))
}

#[test]
fn the_band_is_three_titled_columns_with_the_generation_run_while_ready() {
    // The Recursive Backtracker carves 19 x 6 = 114 cells. The first is
    // marked carved before the first step, each of the other 113 takes one
    // step, and each of the 114 is popped off the stack by one step: 227.
    let app = app_on(80, 24);
    let buffer = draw(&app, 80, 24);

    let border = row(&buffer, 15);
    for title in ["this run", "previous run", "maze"] {
        assert!(border.contains(title), "{title} is in {border:?}");
    }
    assert_eq!(border.matches('┌').count(), 3);
    assert!(row(&buffer, 21).starts_with('└'));

    let [this, previous, maze] = band(&buffer);
    assert_eq!(
        this,
        [
            "algorithm RecBack",
            "step 227",
            "carved 114",
            "frontier 0",
            "path len -"
        ]
    );
    assert_eq!(previous, ["", "", "", "", ""]);
    assert_eq!(
        maze,
        [
            "maze 19x6",
            "braid 0.00",
            "carved 114",
            "start 0,0",
            "equal path"
        ]
    );
}

#[test]
fn a_label_starts_at_the_left_border_and_a_value_ends_at_the_right_border() {
    // Section 8.1 sizes the names against the columns inside the border, so
    // the rows have no padding.
    let app = app_on(80, 24);
    let buffer = draw(&app, 80, 24);
    let text = row(&buffer, 17);
    let cells: Vec<&str> = text.split('│').collect();
    assert!(cells[1].starts_with("step "), "{:?}", cells[1]);
    assert!(cells[1].ends_with(" 227"), "{:?}", cells[1]);
}

#[test]
fn a_solver_run_shows_expanded_and_the_path_length_once_solved() {
    let mut app = app_on(80, 24);
    assert!(app.apply(Action::Solve));
    let buffer = draw(&app, 80, 24);
    let [this, ..] = band(&buffer);
    assert_eq!(
        this,
        [
            "algorithm A*",
            "step 0",
            "expanded 0",
            "frontier 1",
            "path len -"
        ]
    );

    while app.step() == StepOutcome::Stepped {}
    app.advance_phase();
    let Some(stats) = app.run.as_ref().map(|run| run.stats(&app.maze)) else {
        panic!("a solved application holds its solver run");
    };
    let Some(path_len) = stats.path_len else {
        panic!("a solved run has a path");
    };
    let expected = [
        "algorithm A*".to_string(),
        format!("step {}", stats.steps),
        format!("expanded {}", stats.expanded_or_carved),
        format!("frontier {}", stats.frontier),
        format!("path len {path_len}"),
    ];
    let buffer = draw(&app, 80, 24);
    let [this, previous, _] = band(&buffer);
    assert_eq!(this, expected);
    assert_eq!(previous, ["", "", "", "", ""]);

    // Leaving `Solved` promotes the run, and this run is empty until `s`.
    assert!(app.apply(Action::ChooseSolver(1)));
    let buffer = draw(&app, 80, 24);
    let [this, previous, _] = band(&buffer);
    assert_eq!(this, ["", "", "", "", ""]);
    assert_eq!(previous, expected);
}

#[test]
fn a_braided_maze_drops_equal_path() {
    let mut app = app_on(80, 24);
    assert!(app.apply(Action::CycleBraid));
    let buffer = draw(&app, 80, 24);
    let [_, _, maze] = band(&buffer);
    assert_eq!(maze[1], "braid 0.25");
    assert_eq!(maze[4], "");
}

#[test]
fn the_band_holds_the_longest_abbreviated_name_at_the_79_column_floor() {
    let startup = Startup {
        generator_ix: 1,
        width: Some(512),
        height: Some(512),
        ..startup()
    };
    let app = App::new(startup, capacity(79, 19));
    let buffer = draw(&app, 79, 19);
    let [this, _, maze] = band(&buffer);
    assert_eq!(this[0], "algorithm RandPrim");
    assert_eq!(this[2], "carved 262144");
    assert_eq!(maze[0], "maze 512x512");
    assert_eq!(maze[2], "carved 262144");
}

// ---------------------------------------------------------------------------
// The help overlay of section 7.5
// ---------------------------------------------------------------------------

/// The help overlay, border and all: the whole keymap of section 7.5.
///
/// Three notes are shorter than the table in section 7.5, so that the box is
/// 76 columns and fits the 79-column layout floor: `.`, `n` and `b`.
const OVERLAY: [&str; 18] = [
    "┌ keymap ──────────────────────────────────────────────────────────────────┐",
    "│ g        generate               animated, from the current seed          │",
    "│ G        generate instantly     skips the animation                      │",
    "│ s        solve                  runs the selected solver                 │",
    "│ Space    pause / resume         Generating and Solving                   │",
    "│ .        single step            exactly one step at any speed; pauses    │",
    "│ + -      speed up / down        one rung of the ladder                   │",
    "│ 1 2      choose generator       selects; does not generate               │",
    "│ 3 4 5    choose solver          Solved becomes Ready                     │",
    "│ Tab      next solver            shortcut for 3-5, wraps                  │",
    "│ <- ->    maze narrower / wider  regenerates instantly                    │",
    "│ Up Down  maze taller / shorter  regenerates instantly                    │",
    "│ f        refit and regenerate   fits the maze to the terminal now        │",
    "│ n        new seed               fresh random seed; regenerates instantly │",
    "│ b        cycle braid factor     0.00, 0.25, 0.50; regenerates instantly  │",
    "│ ?        help                   the full keymap over the maze pane       │",
    "│ q Esc    quit                   restores the terminal                    │",
    "└───────────────────────────────────────────────────────── ? or Esc closes ┘",
];

/// Holds the whole overlay to the screen, with its top left corner at
/// `(left, top)`.
fn assert_overlay_at(buffer: &Buffer, left: u16, top: u16) {
    for (line, expected) in (top..).zip(OVERLAY) {
        let text: String = row(buffer, line)
            .chars()
            .skip(usize::from(left))
            .take(76)
            .collect();
        assert_eq!(text, expected, "overlay row {line}");
    }
}

#[test]
fn the_help_overlay_is_centred_over_the_maze_pane() {
    let mut app = app_on(120, 30);
    assert!(app.apply(Action::ToggleHelp));
    let buffer = draw(&app, 120, 30);
    // The pane is 120 x 20 from row 1, and the box is 76 x 18.
    assert_overlay_at(&buffer, 22, 2);
}

#[test]
fn a_short_pane_keeps_the_overlay_whole_and_the_status_row_uncovered() {
    // At 80 x 24 the pane is 14 rows, and at the 79 x 19 floor it is 9. The
    // box is 18 rows, so it starts at the top of the pane and covers part of
    // the chrome below. Section 7.2 keeps the seed on screen.
    for (cols, rows, left) in [(80, 24, 2), (79, 19, 1)] {
        let mut app = app_on(cols, rows);
        assert!(app.apply(Action::ToggleHelp));
        let buffer = draw(&app, cols, rows);
        assert_overlay_at(&buffer, left, 1);
        assert!(row(&buffer, 0).starts_with("MazeLab  seed 7  "));
    }
}

#[test]
fn a_closed_overlay_draws_nothing() {
    let mut app = app_on(120, 30);
    assert!(app.apply(Action::ToggleHelp));
    assert!(app.apply(Action::Dismiss));
    let buffer = draw(&app, 120, 30);
    assert_eq!(count(&buffer, "keymap"), 0);
    assert_maze_at(&buffer, 1, 1, 118, 19);
}

// ---------------------------------------------------------------------------
// Item 9 of section 11.3: a full-screen snapshot of a small solved maze
// ---------------------------------------------------------------------------

/// Runs the selected solver from `Ready` to `Solved`, exactly as the loop of
/// section 6.1 does.
fn solve(app: &mut App) {
    assert!(app.apply(Action::Solve));
    while app.step() == StepOutcome::Stepped {}
    app.advance_phase();
}

/// One row of the maze pane at the layout floor, as a styled line.
///
/// The maze is 22 columns wide and the pane is 79, so 28 blank columns stand
/// on its left. Each glyph takes the colour that the solver table of section
/// 7.4 gives its cell state.
fn maze_row(glyphs: &str) -> Line<'static> {
    let bold = |colour| Style::new().fg(colour).add_modifier(Modifier::BOLD);
    let chars: Vec<char> = glyphs.chars().collect();
    let mut spans = vec![Span::raw(" ".repeat(28))];
    for pair in chars.chunks(2) {
        let glyph: String = pair.iter().collect();
        let style = match glyph.as_str() {
            "SS" => bold(Color::LightGreen),
            "GG" => bold(Color::LightRed),
            "@@" => bold(Color::White),
            "▓▓" => Style::new().fg(Color::Green),
            "▒▒" => Style::new().fg(Color::LightCyan),
            "░░" => Style::new().fg(Color::Blue),
            "██" => Style::new().fg(Color::DarkGray),
            "  " => Style::new(),
            other => panic!("{other:?} is not a solver glyph"),
        };
        spans.push(Span::styled(glyph, style));
    }
    Line::from(spans)
}

/// The whole screen at the 79 x 19 layout floor: a 5 x 4 maze at seed 2,
/// solved by DFS, then by BFS, so both run columns are full.
///
/// Worked by hand from the maze on screen, which is a tree of 19 edges. BFS
/// expands (0,0) (1,0) (2,0) (2,1) (2,2) (2,3) (3,3) (1,3) (3,2) (0,3) (4,2)
/// (0,2) (4,1) and then the goal (4,3): 14 steps, 14 expanded, and (1,2) and
/// (3,1) left in the queue. The path is 10 cells. DFS reaches the goal on its
/// 16th step with (4,1) left on the stack, along the same path, because the
/// maze is perfect.
///
/// Every passage takes the weaker of its two ends: the passage between
/// frontier (1,2) and expanded (0,2) is expanded, and so is the one between
/// expanded (1,3) and path (2,3). The goal is also the current cell, and goal
/// ranks above current.
///
/// The legend row and the help row are the last two rows, in their short
/// forms, because 79 columns is below the breakpoint.
#[test]
fn a_small_solved_maze_fills_the_screen_at_the_layout_floor() {
    let startup = Startup {
        seed: 2,
        width: Some(5),
        height: Some(4),
        solver_ix: 0,
        ..startup()
    };
    let mut app = App::new(startup, capacity(79, 19));
    solve(&mut app);
    assert!(app.apply(Action::ChooseSolver(1)));
    solve(&mut app);

    terminal(&app, 79, 19).backend().assert_buffer_lines([
        Line::from(
            "MazeLab  seed 2  gen Recursive Backtracker  solver BFS  phase Solved  speed 8x",
        ),
        maze_row("██████████████████████"),
        maze_row("██SS▓▓▓▓▓▓▓▓██      ██"),
        maze_row("██████████▓▓██  ██████"),
        maze_row("██      ██▓▓██▒▒░░░░██"),
        maze_row("██████  ██▓▓██████░░██"),
        maze_row("██░░░░▒▒██▓▓██▓▓▓▓▓▓██"),
        maze_row("██░░██████▓▓██▓▓██▓▓██"),
        maze_row("██░░░░░░░░▓▓▓▓▓▓██GG██"),
        maze_row("██████████████████████"),
        Line::from(
            "┌ this run ──────────────┐┌ previous run ───────────┐┌ maze ──────────────────┐",
        ),
        Line::from(
            "│algorithm            BFS││algorithm             DFS││maze                 5x4│",
        ),
        Line::from(
            "│step                  14││step                   16││braid               0.00│",
        ),
        Line::from(
            "│expanded              14││expanded               16││carved                20│",
        ),
        Line::from(
            "│frontier               2││frontier                1││start                0,0│",
        ),
        Line::from(
            "│path len              10││path len               10││equal path              │",
        ),
        Line::from(
            "└────────────────────────┘└─────────────────────────┘└────────────────────────┘",
        ),
        legend_row(),
        help_row(),
    ]);
}

/// The short solver legend of section 7.6, in the colours of section 7.4.
fn legend_row() -> Line<'static> {
    let bold = |colour| Style::new().fg(colour).add_modifier(Modifier::BOLD);
    Line::from(vec![
        Span::styled("SS", bold(Color::LightGreen)),
        Span::raw(" start "),
        Span::styled("GG", bold(Color::LightRed)),
        Span::raw(" goal "),
        Span::styled("@@", bold(Color::White)),
        Span::raw(" cur "),
        Span::styled("▓▓", Style::new().fg(Color::Green)),
        Span::raw(" path "),
        Span::styled("▒▒", Style::new().fg(Color::LightCyan)),
        Span::raw(" frontier "),
        Span::styled("░░", Style::new().fg(Color::Blue)),
        Span::raw(" expanded [  ] floor "),
        Span::styled("██", Style::new().fg(Color::DarkGray)),
        Span::raw(" wall"),
    ])
}

/// The short help row of section 7.7, with the keys in `Yellow`.
fn help_row() -> Line<'static> {
    let key = |text| Span::styled(text, Style::new().fg(Color::Yellow));
    Line::from(vec![
        key("g"),
        Span::raw(" generate  "),
        key("s"),
        Span::raw(" solve  "),
        key("Space"),
        Span::raw(" pause  "),
        key("."),
        Span::raw(" step  "),
        key("+/-"),
        Span::raw(" speed  "),
        key("?"),
        Span::raw(" help  "),
        key("q"),
        Span::raw(" quit"),
    ])
}
