//! Items 8 and 9 of tier 1 of the test plan, and the transitions of section 5.1.
//!
//! Item 8 is capacity and the size clamp, which are pure functions of two
//! integers. Item 9 is the arithmetic of [`StepBudget`], fed synthetic
//! durations. Section 11.4 forbids driving the real event loop, so the budget
//! is tested through the one seam that holds the arithmetic, and the loop that
//! measures the time is not tested at all.
//!
//! Every transition of section 5.1 has a test below, and one test holds the
//! rule that section 5.2 states and the diagram cannot carry: an abandoned
//! solver run does not promote to the previous-run column.

use std::time::Duration;

use mazelab::StepOutcome;
use mazelab::app::{
    Action, Activity, App, DEFAULT_RUNG, LADDER, MIN_SIZE, Phase, Startup, StepBudget, TooSmall,
    capacity,
};

/// One frame at the nominal 60 Hz redraw rate of section 6.1.
const FRAME: Duration = Duration::from_nanos(16_666_667);

/// The startup options every test below starts from.
///
/// The seed is fixed, so that a test that regenerates gets the same maze
/// twice, and the solver is A\*, which is the default of section 9.
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

/// An application with a maze on screen, at the capacity of a 120 x 30
/// terminal.
fn app() -> App {
    App::new(startup(), capacity(120, 30))
}

/// An application at the `Idle` phase of section 5.1, before its first maze.
fn idle() -> App {
    App::idle(startup(), capacity(120, 30))
}

/// Runs the current run to its end, exactly as the loop of section 6.1 does.
fn finish(app: &mut App) {
    while app.step() == StepOutcome::Stepped {}
    app.advance_phase();
}

/// Generates a maze and solves it, which is the one way to reach `Solved`.
fn solved() -> App {
    let mut app = app();
    assert!(app.apply(Action::Solve));
    finish(&mut app);
    assert_eq!(app.phase, Phase::Solved);
    app
}

// ---------------------------------------------------------------------------
// Item 8: capacity and the size clamp
// ---------------------------------------------------------------------------

/// The three terminal sizes that section 7.1 tabulates.
#[test]
fn capacity_is_the_table_of_section_7_1() {
    assert_eq!(capacity(120, 30), Some((29, 9)));
    assert_eq!(capacity(80, 24), Some((19, 6)));
    assert_eq!(capacity(79, 19), Some((19, 4)));
}

/// Capacity is undefined below the layout floor, on either axis. Section 13.2
/// sends the startup chain to the 4 x 4 minimum there.
#[test]
fn capacity_is_undefined_below_the_layout_floor() {
    assert_eq!(capacity(78, 19), None);
    assert_eq!(capacity(79, 18), None);
    assert_eq!(capacity(60, 18), None);
    assert_eq!(capacity(0, 0), None);
}

/// Capacity at the floor is 19 x 4, and the floor of the maze is 4 rows, so
/// the two floors of section 13.1 agree.
#[test]
fn capacity_at_the_layout_floor_meets_the_minimum_maze() {
    let Some((w, h)) = capacity(79, 19) else {
        panic!("79 x 19 is the layout floor, so capacity is defined there")
    };
    assert!(w >= MIN_SIZE && h >= MIN_SIZE);
}

/// The startup size is the flag if given, else capacity, else the 4 x 4
/// minimum. Section 13.2 gives the chain, and section 9 uses the same one for
/// a pair where only one flag is given.
#[test]
fn the_startup_size_is_the_fallback_chain_of_section_13_2() {
    let stated = Startup {
        width: Some(40),
        height: Some(7),
        ..startup()
    };
    let app = App::new(stated, capacity(120, 30));
    assert_eq!((app.maze.width(), app.maze.height()), (40, 7));

    let app = App::new(startup(), capacity(120, 30));
    assert_eq!((app.maze.width(), app.maze.height()), (29, 9));

    let app = App::new(startup(), capacity(60, 18));
    assert_eq!((app.maze.width(), app.maze.height()), (4, 4));

    let one_axis = Startup {
        width: Some(40),
        ..startup()
    };
    let app = App::new(one_axis, capacity(120, 30));
    assert_eq!((app.maze.width(), app.maze.height()), (40, 9));
}

/// A stated size is not clamped to capacity. Section 13.2: an explicit
/// `--width 60` on a 40-column terminal is a stated intention.
#[test]
fn a_stated_size_is_not_clamped_to_capacity() {
    let oversized = Startup {
        width: Some(60),
        height: Some(20),
        ..startup()
    };
    let app = App::new(oversized, capacity(80, 24));
    assert_eq!((app.maze.width(), app.maze.height()), (60, 20));
}

/// A size key moves the maze by one cell and regenerates instantly.
#[test]
fn a_size_key_moves_one_cell_and_regenerates_instantly() {
    let roomy = Startup {
        width: Some(10),
        height: Some(6),
        ..startup()
    };
    let mut app = App::new(roomy, capacity(120, 30));

    assert!(app.apply(Action::Wider));
    assert_eq!((app.maze.width(), app.maze.height()), (11, 6));
    assert_eq!(app.phase, Phase::Ready);
    assert_eq!(app.maze.carved_count(), 11 * 6, "the new maze is finished");

    assert!(app.apply(Action::Taller));
    assert_eq!((app.maze.width(), app.maze.height()), (11, 7));
    assert_eq!(app.phase, Phase::Ready);

    assert!(app.apply(Action::Narrower));
    assert!(app.apply(Action::Shorter));
    assert_eq!((app.maze.width(), app.maze.height()), (10, 6));
}

/// At the clamp a size key is silently inert: no new maze, and nothing to
/// redraw. Section 13.2 names the statistics band as the feedback.
///
/// At the layout floor the maze is 19 x 4, which is capacity on both axes and
/// the minimum maze on one, so three of the four keys are at a clamp and the
/// fourth still moves.
#[test]
fn a_size_key_at_the_clamp_is_inert() {
    let mut app = App::new(startup(), capacity(79, 19));
    assert_eq!((app.maze.width(), app.maze.height()), (19, 4));

    assert!(!app.apply(Action::Wider), "wider at capacity");
    assert!(!app.apply(Action::Taller), "taller at capacity");
    assert!(!app.apply(Action::Shorter), "shorter at the minimum maze");
    assert_eq!((app.maze.width(), app.maze.height()), (19, 4));

    assert!(app.apply(Action::Narrower), "narrower has room");
    assert_eq!((app.maze.width(), app.maze.height()), (18, 4));
}

/// The minimum maze of section 13.1 is the floor a shrink key stops at.
#[test]
fn a_shrink_key_stops_at_the_minimum_maze() {
    let mut app = App::new(startup(), capacity(60, 18));
    assert_eq!((app.maze.width(), app.maze.height()), (MIN_SIZE, MIN_SIZE));
    assert!(!app.apply(Action::Narrower));
    assert!(!app.apply(Action::Shorter));
    assert_eq!((app.maze.width(), app.maze.height()), (MIN_SIZE, MIN_SIZE));
}

/// A grow key never shrinks an oversized maze. An oversized `--width` puts the
/// user in the maze-fit panel of section 13.5, and `f` is the way out.
#[test]
fn a_grow_key_never_shrinks_an_oversized_maze() {
    let oversized = Startup {
        width: Some(60),
        height: Some(20),
        ..startup()
    };
    let mut app = App::new(oversized, capacity(80, 24));
    assert!(!app.apply(Action::Wider));
    assert!(!app.apply(Action::Taller));
    assert_eq!((app.maze.width(), app.maze.height()), (60, 20));

    assert!(app.apply(Action::Refit));
    assert_eq!((app.maze.width(), app.maze.height()), (19, 6));
}

/// Below the layout floor `f` is inert. Section 13.3: clamping to 4 x 4 and
/// regenerating would change nothing visible, because the chrome cannot
/// render.
#[test]
fn refit_below_the_layout_floor_is_inert() {
    let mut app = App::new(startup(), capacity(60, 18));
    assert!(!app.apply(Action::Refit));
    assert_eq!((app.maze.width(), app.maze.height()), (4, 4));
}

/// Capacity is learned from the caller, and the clamp follows it.
#[test]
fn the_clamp_follows_the_capacity_the_caller_gives() {
    let mut app = App::new(startup(), capacity(79, 19));
    assert!(!app.apply(Action::Wider));

    app.set_capacity(capacity(120, 30));
    assert!(app.apply(Action::Wider));
    assert_eq!(app.maze.width(), 20);
}

// ---------------------------------------------------------------------------
// Item 9: StepBudget arithmetic
// ---------------------------------------------------------------------------

/// The ladder and the default rung of section 6.4.
#[test]
fn the_ladder_is_the_one_of_section_6_4() {
    let stated = [0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 64.0, 256.0, 1024.0];
    for (rung, want) in LADDER.into_iter().zip(stated) {
        assert!(
            (rung - want).abs() < f64::EPSILON,
            "the ladder moved: {rung}"
        );
    }
    assert!((StepBudget::new().rung() - LADDER[DEFAULT_RUNG]).abs() < f64::EPSILON);
    assert!((StepBudget::new().rung() - 8.0).abs() < f64::EPSILON);
}

/// A budget at the bottom rung.
fn slowest() -> StepBudget {
    let mut budget = StepBudget::new();
    for _ in 0..DEFAULT_RUNG {
        budget.slower();
    }
    assert!((budget.rung() - 0.5).abs() < f64::EPSILON);
    budget
}

/// At the 0.5 rung one frame yields no step and the next frame yields one.
/// This is the arithmetic item 9 of the test plan names.
#[test]
fn at_the_bottom_rung_one_frame_yields_no_step_and_the_next_yields_one() {
    let mut budget = slowest();
    assert_eq!(budget.advance(FRAME), 0);
    assert_eq!(budget.advance(FRAME), 1);
    assert_eq!(budget.advance(FRAME), 0);
    assert_eq!(budget.advance(FRAME), 1);
}

/// At the default rung one frame yields the rung.
#[test]
fn at_the_default_rung_one_frame_yields_eight_steps() {
    let mut budget = StepBudget::new();
    assert_eq!(budget.advance(FRAME), 8);
}

/// The budget follows the clock and not a count of ticks: two ticks of half
/// the time buy the steps one tick of the whole time buys.
#[test]
fn the_budget_follows_the_clock_and_not_the_tick() {
    let mut whole = StepBudget::new();
    assert_eq!(whole.advance(Duration::from_millis(50)), 24);

    let mut halves = StepBudget::new();
    assert_eq!(halves.advance(Duration::from_millis(25)), 12);
    assert_eq!(halves.advance(Duration::from_millis(25)), 12);
}

/// `discard` zeroes the residue, which is what section 6.3 has the loop do at
/// the deadline. Without the discard the two calls below buy one step.
#[test]
fn discard_zeroes_the_residue() {
    let mut budget = StepBudget::new();
    for _ in 0..3 {
        budget.slower();
    }
    assert!((budget.rung() - 1.0).abs() < f64::EPSILON);

    assert_eq!(budget.advance(Duration::from_millis(10)), 0);
    budget.discard();
    assert_eq!(budget.advance(Duration::from_millis(10)), 0);
}

/// `reset` zeroes the residue, which is what a resume and a single step do.
#[test]
fn reset_zeroes_the_residue() {
    let mut budget = StepBudget::new();
    for _ in 0..3 {
        budget.slower();
    }
    assert_eq!(budget.advance(Duration::from_millis(10)), 0);
    budget.reset();
    assert_eq!(budget.advance(Duration::from_millis(10)), 0);
}

/// A speed change keeps the residue. Section 6.4: the residue is under one
/// step by definition, so dropping it is unobservable, and a rule with no
/// effect is a rule to get wrong later.
#[test]
fn a_speed_change_keeps_the_residue() {
    let mut budget = slowest();
    // 0.3 of a step at the 0.5 rung.
    assert_eq!(budget.advance(Duration::from_millis(10)), 0);

    budget.faster();
    assert!((budget.rung() - 1.0).abs() < f64::EPSILON);
    // 0.72 of a step at the 1.0 rung. The two together reach one step, and
    // only a kept residue can.
    assert_eq!(budget.advance(Duration::from_millis(12)), 1);
}

/// The rung saturates at both ends of the ladder.
#[test]
fn the_rung_saturates_at_both_ends_of_the_ladder() {
    let mut budget = StepBudget::new();
    for _ in 0..20 {
        budget.faster();
    }
    assert!((budget.rung() - 1024.0).abs() < f64::EPSILON);

    for _ in 0..20 {
        budget.slower();
    }
    assert!((budget.rung() - 0.5).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// The transitions of section 5.1
// ---------------------------------------------------------------------------

/// `Idle --g--> Generating`.
#[test]
fn idle_generates_animated_on_g() {
    let mut app = idle();
    assert_eq!(app.phase, Phase::Idle);
    assert!(app.apply(Action::Generate));
    assert_eq!(app.phase, Phase::Generating);
    assert!(app.run.is_some());
}

/// `Idle --G--> Ready`. The `Generating` phase is never entered, and the maze
/// is finished, which is section 6.5.
#[test]
fn idle_generates_instantly_on_capital_g() {
    let mut app = idle();
    assert!(app.apply(Action::GenerateInstantly));
    assert_eq!(app.phase, Phase::Ready);
    assert_eq!(app.maze.carved_count(), 29 * 9);
}

/// `Generating <--Space--> Paused(Generating)`.
#[test]
fn space_pauses_and_resumes_a_generation() {
    let mut app = idle();
    app.apply(Action::Generate);
    assert!(app.apply(Action::PauseResume));
    assert_eq!(app.phase, Phase::Paused(Activity::Generating));
    assert!(app.apply(Action::PauseResume));
    assert_eq!(app.phase, Phase::Generating);
}

/// `Generating --Done--> Ready`, after the braiding pass.
#[test]
fn a_finished_generation_goes_to_ready() {
    let mut app = idle();
    app.apply(Action::Generate);
    finish(&mut app);
    assert_eq!(app.phase, Phase::Ready);
    assert_eq!(app.maze.carved_count(), 29 * 9);
}

/// `Ready --s--> Solving`.
#[test]
fn ready_solves_on_s() {
    let mut app = app();
    assert_eq!(app.phase, Phase::Ready);
    assert!(app.apply(Action::Solve));
    assert_eq!(app.phase, Phase::Solving);
}

/// `Solving <--Space--> Paused(Solving)`.
#[test]
fn space_pauses_and_resumes_a_solve() {
    let mut app = app();
    app.apply(Action::Solve);
    assert!(app.apply(Action::PauseResume));
    assert_eq!(app.phase, Phase::Paused(Activity::Solving));
    assert!(app.apply(Action::PauseResume));
    assert_eq!(app.phase, Phase::Solving);
}

/// `Solving --Done--> Solved`.
#[test]
fn a_finished_solve_goes_to_solved() {
    let app = solved();
    let Some(run) = app.run.as_ref() else {
        panic!("a solved run is still the run on screen")
    };
    assert!(run.stats(&app.maze).path_len.is_some());
}

/// `Solved --3 4 5 Tab--> Ready`, and the finished run promotes to the
/// previous-run column. Section 8.2 fills the column when the phase **leaves**
/// `Solved`.
#[test]
fn choosing_a_solver_from_solved_goes_to_ready_and_promotes() {
    for action in [
        Action::ChooseSolver(0),
        Action::ChooseSolver(1),
        Action::ChooseSolver(2),
        Action::NextSolver,
    ] {
        let mut app = solved();
        assert!(app.previous.is_none());
        assert!(app.apply(action));
        assert_eq!(app.phase, Phase::Ready);
        let Some(previous) = app.previous else {
            panic!("a finished run promotes")
        };
        assert_eq!(previous.algorithm_short, "A*");
        assert!(previous.path_len.is_some());
        assert!(
            app.run.is_none(),
            "the promoted run left the this-run column"
        );
    }
}

/// Choosing the solver that is already selected promotes too, so `s` after it
/// re-runs the same solver against the previous run.
#[test]
fn choosing_the_selected_solver_from_solved_promotes_too() {
    let mut app = solved();
    let selected = app.solver_ix;
    assert!(app.apply(Action::ChooseSolver(selected)));
    assert_eq!(app.phase, Phase::Ready);
    assert!(app.previous.is_some());
    assert_eq!(app.solver_ix, selected);

    assert!(app.apply(Action::Solve));
    assert_eq!(app.phase, Phase::Solving);
}

/// `Tab` wraps, which is the one cycling shortcut of section 7.5.
#[test]
fn tab_cycles_the_solver_and_wraps() {
    let mut app = app();
    app.solver_ix = 0;
    app.apply(Action::NextSolver);
    assert_eq!(app.solver_ix, 1);
    app.apply(Action::NextSolver);
    assert_eq!(app.solver_ix, 2);
    app.apply(Action::NextSolver);
    assert_eq!(app.solver_ix, 0);
}

/// **An abandoned solver run does not promote.** Section 5.2: it is
/// incomplete, and a half-finished expanded count beside a finished one would
/// be read as a comparison.
#[test]
fn an_abandoned_solver_run_does_not_promote() {
    for pause_first in [false, true] {
        let mut app = app();
        app.apply(Action::Solve);
        app.step();
        app.step();
        if pause_first {
            app.apply(Action::PauseResume);
            assert_eq!(app.phase, Phase::Paused(Activity::Solving));
        }

        assert!(app.apply(Action::ChooseSolver(1)));
        assert_eq!(app.phase, Phase::Ready);
        assert!(
            app.previous.is_none(),
            "an abandoned run reached the previous-run column"
        );
        assert!(app.run.is_none(), "an abandoned run is still on screen");
        assert_eq!(app.solver_ix, 1);
    }
}

/// A run that promoted once is the only one in the column: the column holds at
/// most one finished solver run.
#[test]
fn the_previous_run_column_holds_the_last_finished_solver_run() {
    let mut app = solved();
    app.apply(Action::ChooseSolver(1));
    assert_eq!(
        app.previous.map(|s| s.algorithm_short),
        Some("A*"),
        "the first solver promoted"
    );

    app.apply(Action::Solve);
    finish(&mut app);
    app.apply(Action::ChooseSolver(0));
    assert_eq!(
        app.previous.map(|s| s.algorithm_short),
        Some("BFS"),
        "the second solver replaced the first"
    );
}

/// `any --g G n b <- -> Up Down f--> Generating or Ready`. Only `g` animates.
#[test]
fn every_new_maze_key_builds_a_maze_from_any_phase() {
    let animated = [Action::Generate];
    let instant = [
        Action::GenerateInstantly,
        Action::NewSeed,
        Action::CycleBraid,
        Action::Narrower,
        Action::Wider,
        Action::Taller,
        Action::Shorter,
        Action::Refit,
    ];

    for action in animated.into_iter().chain(instant) {
        for phase in [
            Phase::Idle,
            Phase::Generating,
            Phase::Paused(Activity::Generating),
            Phase::Ready,
            Phase::Solving,
            Phase::Paused(Activity::Solving),
            Phase::Solved,
        ] {
            let mut app = at_phase(phase);
            assert!(app.apply(action), "{action:?} from {phase:?} did nothing");
            let expected = if action == Action::Generate {
                Phase::Generating
            } else {
                Phase::Ready
            };
            assert_eq!(app.phase, expected, "{action:?} from {phase:?}");
            assert!(app.previous.is_none(), "{action:?} kept the previous run");
        }
    }
}

/// An application in the phase named, with a maze that has room to grow on
/// both axes, so that a size key is not at its clamp.
///
/// The previous-run column is filled wherever a phase can have one, so that a
/// new maze has something to clear.
fn at_phase(phase: Phase) -> App {
    let roomy = Startup {
        width: Some(10),
        height: Some(6),
        ..startup()
    };
    let idle = || App::idle(roomy, capacity(120, 30));
    let ready = || App::new(roomy, capacity(120, 30));
    let solved = || {
        let mut app = ready();
        app.apply(Action::Solve);
        finish(&mut app);
        assert_eq!(app.phase, Phase::Solved);
        app
    };

    match phase {
        Phase::Idle => idle(),
        Phase::Generating | Phase::Paused(Activity::Generating) => {
            let mut app = idle();
            app.apply(Action::Generate);
            app.step();
            if phase == Phase::Paused(Activity::Generating) {
                app.apply(Action::PauseResume);
            }
            app
        }
        Phase::Ready => {
            let mut app = solved();
            app.apply(Action::ChooseSolver(1));
            assert!(app.previous.is_some());
            app
        }
        Phase::Solving | Phase::Paused(Activity::Solving) => {
            let mut app = ready();
            app.apply(Action::Solve);
            app.step();
            if phase == Phase::Paused(Activity::Solving) {
                app.apply(Action::PauseResume);
            }
            app
        }
        Phase::Solved => solved(),
    }
}

// ---------------------------------------------------------------------------
// The rules of section 5.2 that the diagram does not carry
// ---------------------------------------------------------------------------

/// Choosing a generator selects and does not generate. The maze on screen is
/// not touched.
#[test]
fn choosing_a_generator_does_not_generate() {
    let mut app = app();
    let before = app.maze.display_grid();
    assert!(app.apply(Action::ChooseGenerator(1)));
    assert_eq!(app.generator_ix, 1);
    assert_eq!(app.phase, Phase::Ready);
    assert!(
        app.maze.display_grid() == before,
        "choosing a generator rebuilt the maze"
    );
}

/// A key that names an entry the registry does not hold changes nothing.
#[test]
fn an_index_outside_the_registry_is_inert() {
    let mut app = app();
    assert!(!app.apply(Action::ChooseGenerator(9)));
    assert!(!app.apply(Action::ChooseSolver(9)));
    assert_eq!(app.generator_ix, 0);
    assert_eq!(app.solver_ix, 2);
}

/// Single step from a running phase implies pause, and advances exactly one
/// step.
#[test]
fn single_step_pauses_and_advances_one_step() {
    let mut app = idle();
    app.apply(Action::Generate);
    let before = steps(&app);

    assert!(app.apply(Action::SingleStep));
    assert_eq!(app.phase, Phase::Paused(Activity::Generating));
    assert_eq!(steps(&app), before + 1);

    assert!(app.apply(Action::SingleStep));
    assert_eq!(app.phase, Phase::Paused(Activity::Generating));
    assert_eq!(steps(&app), before + 2);
}

/// Single step from a phase with no run does nothing.
#[test]
fn single_step_from_a_still_phase_does_nothing() {
    let mut app = app();
    assert!(!app.apply(Action::SingleStep));
    assert_eq!(app.phase, Phase::Ready);
}

/// The step count of the run on screen.
fn steps(app: &App) -> u64 {
    app.run.as_ref().map_or(0, |run| run.stats(&app.maze).steps)
}

/// A single step that finishes the run advances the phase, so a generation
/// stepped to its end reaches `Ready` like any other.
#[test]
fn a_single_step_that_finishes_the_run_advances_the_phase() {
    let mut app = idle();
    app.apply(Action::Generate);
    while app.phase != Phase::Ready {
        app.apply(Action::SingleStep);
    }
    assert_eq!(app.maze.carved_count(), 29 * 9);
}

/// The speed keys move one rung and change no phase.
#[test]
fn the_speed_keys_move_one_rung() {
    let mut app = app();
    assert!(app.apply(Action::Faster));
    assert!((app.budget.rung() - 16.0).abs() < f64::EPSILON);
    assert!(app.apply(Action::Slower));
    assert!((app.budget.rung() - 8.0).abs() < f64::EPSILON);
    assert_eq!(app.phase, Phase::Ready);
}

/// `b` cycles the braid factor through 0.00, 0.25 and 0.50, and regenerates.
#[test]
fn the_braid_key_cycles_three_factors_and_regenerates() {
    let mut app = app();
    for expected in [0.25, 0.50, 0.00, 0.25] {
        assert!(app.apply(Action::CycleBraid));
        assert!((app.braid_factor - expected).abs() < 1e-9);
        assert_eq!(app.phase, Phase::Ready);
    }
}

/// `n` takes a fresh seed and builds a maze from it.
#[test]
fn the_new_seed_key_takes_a_fresh_seed() {
    let mut app = app();
    let before = app.seed;
    assert!(app.apply(Action::NewSeed));
    assert_ne!(app.seed, before, "the seed did not change");
    assert_eq!(app.phase, Phase::Ready);
    assert_eq!(app.maze.carved_count(), 29 * 9);
}

/// The same seed at the same size gives the same maze, so a regenerate that
/// keeps the seed is reproducible.
#[test]
fn a_regenerate_at_the_same_seed_gives_the_same_maze() {
    let mut app = app();
    let first = app.maze.display_grid();
    app.apply(Action::GenerateInstantly);
    assert!(
        app.maze.display_grid() == first,
        "the same seed at the same size gave a different maze"
    );
}

// ---------------------------------------------------------------------------
// Section 13.4: auto-pause, and never auto-resume
// ---------------------------------------------------------------------------

/// Entering too small auto-pauses the run, on either threshold.
#[test]
fn entering_too_small_auto_pauses_the_run() {
    for threshold in [TooSmall::MazeFit, TooSmall::LayoutFloor] {
        let mut app = app();
        app.apply(Action::Solve);
        app.set_too_small(Some(threshold));
        assert_eq!(app.phase, Phase::Paused(Activity::Solving));

        let mut app = idle();
        app.apply(Action::Generate);
        app.set_too_small(Some(threshold));
        assert_eq!(app.phase, Phase::Paused(Activity::Generating));
    }
}

/// Growing the terminal back does not auto-resume. Section 13.4 gives the
/// reason: dragging a window edge would fire a stutter of pause and resume.
#[test]
fn growing_the_terminal_back_does_not_auto_resume() {
    let mut app = app();
    app.apply(Action::Solve);
    app.set_too_small(Some(TooSmall::LayoutFloor));
    app.set_too_small(None);
    assert_eq!(app.phase, Phase::Paused(Activity::Solving));

    assert!(app.apply(Action::PauseResume));
    assert_eq!(app.phase, Phase::Solving);
}

/// Too small changes no still phase.
#[test]
fn too_small_changes_no_still_phase() {
    let mut app = app();
    app.set_too_small(Some(TooSmall::MazeFit));
    assert_eq!(app.phase, Phase::Ready);
    assert_eq!(app.too_small, Some(TooSmall::MazeFit));
}

// ---------------------------------------------------------------------------
// The help overlay and the quit keys of section 7.5
// ---------------------------------------------------------------------------

/// `?` toggles the overlay and changes no phase, so an animation continues
/// behind it.
#[test]
fn the_help_key_toggles_the_overlay_and_changes_no_phase() {
    let mut app = idle();
    app.apply(Action::Generate);
    assert!(app.apply(Action::ToggleHelp));
    assert!(app.help_open);
    assert_eq!(app.phase, Phase::Generating);
    assert!(app.apply(Action::ToggleHelp));
    assert!(!app.help_open);
}

/// `Esc` closes the overlay instead of quitting, and quits when it is closed.
#[test]
fn escape_closes_the_overlay_before_it_quits() {
    let mut app = app();
    app.apply(Action::ToggleHelp);
    assert!(app.apply(Action::Dismiss));
    assert!(!app.help_open);
    assert!(!app.quit);

    assert!(app.apply(Action::Dismiss));
    assert!(app.quit);
}

/// `q` quits, and the overlay does not hold it back: every other key acts
/// normally while the overlay is open.
#[test]
fn q_quits_with_the_overlay_open() {
    let mut app = app();
    app.apply(Action::ToggleHelp);
    assert!(app.apply(Action::Quit));
    assert!(app.quit);
}
