//! Item 6 of tier 3 of the test plan: the legend row of section 7.6 never
//! drops an entry.
//!
//! The legend is the in-application evidence that the glyphs and the colours
//! arrived intact, so these tests hold every entry, its glyph, its colour and
//! its label. ADR 0007.

use mazelab::StepOutcome;
use mazelab::app::{Action, Activity, App, Phase, Startup, capacity};
use mazelab::ui::{self, legend, palette};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::{Color, Modifier};

/// The text of a line, with no style.
fn text(line: &ratatui::text::Line) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

#[test]
fn the_short_solver_legend_measures_74_columns() {
    let line = legend::line(Activity::Solving, &palette::UNICODE, 79);
    assert_eq!(
        text(&line),
        "SS start GG goal @@ cur ▓▓ path ▒▒ frontier ░░ expanded [  ] floor ██ wall"
    );
    assert_eq!(line.width(), 74);
}

#[test]
fn at_100_columns_the_solver_legend_spells_current_and_measures_85() {
    let line = legend::line(Activity::Solving, &palette::UNICODE, 100);
    assert_eq!(
        text(&line),
        "SS start  GG goal  @@ current  ▓▓ path  ▒▒ frontier  ░░ expanded  [  ] floor  ██ wall"
    );
    assert_eq!(line.width(), 85);
}

#[test]
fn a_generation_run_has_a_legend_of_its_four_cell_states() {
    let short = legend::line(Activity::Generating, &palette::ASCII, 79);
    assert_eq!(text(&short), "@@ cur :: frontier [  ] carved ## uncarved");
    let full = legend::line(Activity::Generating, &palette::ASCII, 100);
    assert_eq!(
        text(&full),
        "@@ current  :: frontier  [  ] carved  ## uncarved"
    );
}

// ---------------------------------------------------------------------------
// Item 6 of section 11.3, on the screen
// ---------------------------------------------------------------------------

/// One legend entry as section 7.4 and section 7.6 give it: the glyph in each
/// glyph set, the colour, whether it is bold, the full label and the short
/// label.
#[derive(Clone, Copy)]
struct Entry {
    unicode: &'static str,
    ascii: &'static str,
    colour: Color,
    bold: bool,
    full: &'static str,
    short: &'static str,
}

/// An entry that is not bold.
const fn entry(
    unicode: &'static str,
    ascii: &'static str,
    colour: Color,
    full: &'static str,
    short: &'static str,
) -> Entry {
    Entry {
        unicode,
        ascii,
        colour,
        bold: false,
        full,
        short,
    }
}

/// The same entry, in bold.
const fn bold(entry: Entry) -> Entry {
    Entry {
        bold: true,
        ..entry
    }
}

/// The solver legend, from the tables of sections 7.4 and 7.6.
const SOLVER: [Entry; 8] = [
    bold(entry("SS", "SS", Color::LightGreen, "start", "start")),
    bold(entry("GG", "GG", Color::LightRed, "goal", "goal")),
    bold(entry("@@", "@@", Color::White, "current", "cur")),
    entry("▓▓", "**", Color::Green, "path", "path"),
    entry("▒▒", "::", Color::LightCyan, "frontier", "frontier"),
    entry("░░", "..", Color::Blue, "expanded", "expanded"),
    entry("  ", "  ", Color::Reset, "floor", "floor"),
    entry("██", "##", Color::DarkGray, "wall", "wall"),
];

/// The generation legend, from the tables of sections 7.4 and 7.6.
const GENERATION: [Entry; 4] = [
    bold(entry("@@", "@@", Color::White, "current", "cur")),
    entry("▒▒", "::", Color::LightMagenta, "frontier", "frontier"),
    entry("  ", "  ", Color::Reset, "carved", "carved"),
    entry("██", "##", Color::DarkGray, "uncarved", "uncarved"),
];

/// The application on a terminal of `cols` x 24, in the glyph set that
/// `ascii` selects, in each of the six phases, with the legend each one
/// shows.
///
/// A phase that draws a generation run shows the generation legend. Every
/// other phase draws with the solver cell states and shows the solver legend.
fn apps_in_every_phase(ascii: bool, cols: u16) -> Vec<(App, &'static [Entry])> {
    let startup = Startup {
        seed: 7,
        width: None,
        height: None,
        ascii,
        generator_ix: 0,
        solver_ix: 2,
    };
    let capacity = capacity(cols, 24);
    let idle = App::idle(startup, capacity);
    let ready = App::new(startup, capacity);
    let with = |actions: &[Action]| {
        let mut app = App::new(startup, capacity);
        for &action in actions {
            assert!(app.apply(action));
        }
        app
    };
    let generating = with(&[Action::Generate]);
    let paused_generating = with(&[Action::Generate, Action::PauseResume]);
    let solving = with(&[Action::Solve]);
    let paused_solving = with(&[Action::Solve, Action::PauseResume]);
    let mut solved = with(&[Action::Solve]);
    while solved.step() == StepOutcome::Stepped {}
    solved.advance_phase();
    for (app, phase) in [
        (&idle, Phase::Idle),
        (&ready, Phase::Ready),
        (&generating, Phase::Generating),
        (&paused_generating, Phase::Paused(Activity::Generating)),
        (&solving, Phase::Solving),
        (&paused_solving, Phase::Paused(Activity::Solving)),
        (&solved, Phase::Solved),
    ] {
        assert_eq!(app.phase, phase);
    }
    vec![
        (idle, &SOLVER),
        (ready, &SOLVER),
        (generating, &GENERATION),
        (paused_generating, &GENERATION),
        (solving, &SOLVER),
        (paused_solving, &SOLVER),
        (solved, &SOLVER),
    ]
}

/// Holds the legend row of one frame to every entry of `entries`, in order,
/// each in its glyph, its colour, its weight and the label of the form for
/// `cols`.
fn assert_legend(app: &App, cols: u16, entries: &[Entry]) {
    // `TestBackend::Error` is `Infallible`, so the two patterns are
    // irrefutable.
    let Ok(mut terminal) = Terminal::new(TestBackend::new(cols, 24));
    let Ok(_) = terminal.draw(|frame| ui::render(frame, app));
    let buffer = terminal.backend().buffer();
    let symbols: Vec<&str> = (0..cols).map(|x| buffer[(x, 22)].symbol()).collect();
    let drawn = symbols.concat();

    let separator = if cols >= 100 { "  " } else { " " };
    let mut expected = String::new();
    // The screen column the next entry starts at. Every glyph here is one
    // column per character.
    let mut x: u16 = 0;
    for entry in entries {
        if !expected.is_empty() {
            expected.push_str(separator);
            x += 1 + u16::from(cols >= 100);
        }
        let glyph = if app.ascii {
            entry.ascii
        } else {
            entry.unicode
        };
        let label = if cols >= 100 { entry.full } else { entry.short };
        let swatch = if glyph == "  " {
            format!("[{glyph}]")
        } else {
            glyph.to_string()
        };
        let at = if glyph == "  " { x + 1 } else { x };
        for column in [at, at + 1] {
            assert_eq!(
                buffer[(column, 22)].fg,
                entry.colour,
                "the colour of {label} at {cols} columns in {:?}",
                app.phase
            );
            assert_eq!(
                buffer[(column, 22)].modifier.contains(Modifier::BOLD),
                entry.bold,
                "the weight of {label} at {cols} columns in {:?}",
                app.phase
            );
        }
        let text = format!("{swatch} {label}");
        x += text.chars().count() as u16;
        expected.push_str(&text);
    }
    assert_eq!(
        drawn.trim_end(),
        expected,
        "the legend at {cols} columns in {:?}",
        app.phase
    );
}

#[test]
fn the_legend_drops_no_entry_at_100_and_79_columns_in_both_glyph_sets_and_every_phase() {
    for cols in [100, 79] {
        for ascii in [false, true] {
            for (app, entries) in apps_in_every_phase(ascii, cols) {
                assert_legend(&app, cols, entries);
            }
        }
    }
}
