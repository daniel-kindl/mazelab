//! Items 1 and 2 of tier 3 of the test plan: the two cell-priority functions.
//!
//! Section 11.3 groups these with the rendering tests, and they need no
//! terminal, because a **cell state** is not a colour. That is the whole point
//! of section 7.3: the priority walk lives in `run.rs`, and `ui/palette.rs`
//! only maps the state it returns to a glyph and a colour.
//!
//! Every test below builds its maze by hand, so the state of a solver part-way
//! through a run is worked out from the maze and not read back out of the
//! solver.

use mazelab::StepOutcome;
use mazelab::generator::{GENERATORS, GeneratorEntry};
use mazelab::maze::{Cell, Dir, DisplayCell, Maze};
use mazelab::rng;
use mazelab::run::{GenCellState, Run, SolverCellState};
use mazelab::solver::{SOLVERS, SolverEntry};

/// The maze every solver test below searches, carved by hand.
///
/// Eight edges over nine cells, so it is a tree and there is one path from
/// `(0, 0)` to `(2, 2)`:
///
/// ```text
///   (0,0)-(1,0)-(2,0)
///     |     |     |
///   (0,1) (1,1) (2,1)
///     |
///   (0,2)-(1,2)-(2,2)
/// ```
fn hand_carved() -> Maze {
    let mut maze = Maze::new(3, 3);
    let mut carve = |a: (u16, u16), b: (u16, u16)| {
        maze.carve(Cell { x: a.0, y: a.1 }, Cell { x: b.0, y: b.1 });
    };
    carve((0, 0), (1, 0));
    carve((1, 0), (2, 0));
    carve((2, 0), (2, 1));
    carve((1, 0), (1, 1));
    carve((0, 0), (0, 1));
    carve((0, 1), (0, 2));
    carve((0, 2), (1, 2));
    carve((1, 2), (2, 2));
    maze
}

/// The generator registry entry with this key.
fn generator_named(key: &str) -> &'static GeneratorEntry {
    GENERATORS
        .iter()
        .find(|e| e.key == key)
        .unwrap_or_else(|| panic!("no generator answers to {key}"))
}

/// The solver registry entry with this key.
fn solver_named(key: &str) -> &'static SolverEntry {
    SOLVERS
        .iter()
        .find(|e| e.key == key)
        .unwrap_or_else(|| panic!("no solver answers to {key}"))
}

/// A BFS run over the hand-carved maze, advanced `steps` times.
///
/// BFS draws no random number and walks the neighbours of a cell in the fixed
/// `North, East, South, West` order, so the state after a fixed number of
/// steps is fixed too. Three steps leave `(0, 0)`, `(1, 0)` and `(0, 1)`
/// expanded, `(2, 0)`, `(1, 1)` and `(0, 2)` in the queue, and `(0, 1)`
/// current.
fn bfs_after(maze: &mut Maze, steps: u32) -> Run {
    let mut run = Run::solving(maze, solver_named("bfs"));
    advance(&mut run, maze, steps, "BFS");
    run
}

/// Advances a run `steps` times, and holds it to a run that is still going.
///
/// A run that finished early would leave every assertion after it reading a
/// state that no test wrote down, so the name of the algorithm is carried here
/// to say which run stopped.
fn advance(run: &mut Run, maze: &mut Maze, steps: u32, name: &str) {
    for taken in 0..steps {
        assert_eq!(
            run.step(maze),
            StepOutcome::Stepped,
            "{name} finished after {taken} of the {steps} steps this test takes"
        );
    }
}

/// The display position of a maze cell, which section 2.3 puts at
/// `(2x + 1, 2y + 1)`.
const fn at(x: u16, y: u16) -> (u16, u16) {
    (2 * x + 1, 2 * y + 1)
}

/// The wall position between two adjacent cells, given the display position of
/// each, which is the position halfway between the two.
const fn between(a: (u16, u16), b: (u16, u16)) -> (u16, u16) {
    (u16::midpoint(a.0, b.0), u16::midpoint(a.1, b.1))
}

/// Item 1: one case per solver cell state, from one run.
///
/// The two cells that match more than one condition at once are the point of
/// the test. `(0, 0)` is the start and is expanded, and `(0, 1)` is current
/// and is expanded. Each shows the earlier of the states it matches.
#[test]
fn the_solver_walk_takes_the_first_condition_a_cell_matches() {
    let mut maze = hand_carved();
    let run = bfs_after(&mut maze, 3);
    let state = |x: u16, y: u16| {
        let (dx, dy) = at(x, y);
        run.solver_state(&maze, dx, dy)
    };

    assert_eq!(state(0, 0), SolverCellState::Start, "start, and expanded");
    assert_eq!(state(2, 2), SolverCellState::Goal, "the goal, untouched");
    assert_eq!(
        state(0, 1),
        SolverCellState::Current,
        "current, and expanded"
    );
    assert_eq!(state(1, 0), SolverCellState::Expanded, "expanded only");
    assert_eq!(state(2, 0), SolverCellState::Frontier, "in the queue");
    assert_eq!(state(2, 1), SolverCellState::Open, "no state at all");
    assert_eq!(
        run.solver_state(&maze, 0, 0),
        SolverCellState::Wall,
        "the corner of the display grid is a post"
    );
}

/// A BFS run over the hand-carved maze, advanced until it reached the goal.
///
/// The run takes nine steps, so every cell is expanded and the path holds
/// `(0, 0)`, `(0, 1)`, `(0, 2)`, `(1, 2)` and `(2, 2)`.
fn bfs_solved(maze: &mut Maze) -> Run {
    let mut run = Run::solving(maze, solver_named("bfs"));
    for step in 1..=9 {
        let outcome = run.step(maze);
        let expected = if step == 9 {
            StepOutcome::Done
        } else {
            StepOutcome::Stepped
        };
        assert_eq!(
            outcome, expected,
            "BFS reaches the goal on step 9, and this is the outcome of step {step}"
        );
    }
    run
}

/// Item 2: a passage takes the weaker of its two ends.
///
/// The passage between two path cells is `Path`, so a solved path reads as one
/// connected line. The passage between two expanded cells is `Expanded`, so an
/// expanded region reads as a region. The passage between a frontier cell and
/// an expanded one is `Expanded`, which is what keeps the frontier a thin ring.
#[test]
fn a_passage_takes_the_weaker_of_its_two_ends() {
    let mut maze = hand_carved();
    let run = bfs_after(&mut maze, 3);

    // `(1, 0)` is expanded and `(1, 1)` is in the queue. The weaker of the two
    // is `Expanded`, and this is the case that holds the frontier to a ring.
    let (dx, dy) = between(at(1, 0), at(1, 1));
    assert_eq!(
        run.solver_state(&maze, dx, dy),
        SolverCellState::Expanded,
        "a frontier cell beside an expanded one"
    );
    // `(0, 2)` is in the queue and `(1, 2)` is untouched.
    let (dx, dy) = between(at(0, 2), at(1, 2));
    assert_eq!(
        run.solver_state(&maze, dx, dy),
        SolverCellState::Open,
        "a frontier cell beside a cell the run has not reached"
    );

    let mut maze = hand_carved();
    let run = bfs_solved(&mut maze);
    let state_between = |a: (u16, u16), b: (u16, u16)| {
        let (dx, dy) = between(at(a.0, a.1), at(b.0, b.1));
        run.solver_state(&maze, dx, dy)
    };
    assert_eq!(
        state_between((0, 1), (0, 2)),
        SolverCellState::Path,
        "the passage between `(0, 1)` and `(0, 2)`, both on the path"
    );
    assert_eq!(
        state_between((1, 0), (1, 1)),
        SolverCellState::Expanded,
        "the passage between `(1, 0)` and `(1, 1)`, both expanded, neither on the path"
    );
}

/// The two ends of a passage resolve to their non-positional state, so the
/// walk skips `Start`, `Goal` and `Current` and continues at `Path`.
///
/// Without the skip, the passage out of the start cell would draw as a second
/// start, and the line the path draws would break at three places.
#[test]
fn a_passage_beside_the_start_the_goal_or_the_current_cell_is_path() {
    let mut maze = hand_carved();
    let run = bfs_solved(&mut maze);
    let state_between = |a: (u16, u16), b: (u16, u16)| {
        let (dx, dy) = between(at(a.0, a.1), at(b.0, b.1));
        run.solver_state(&maze, dx, dy)
    };

    assert_eq!(
        state_between((0, 0), (0, 1)),
        SolverCellState::Path,
        "the passage between the start and `(0, 1)`"
    );
    // The goal is the cell the last step expanded, so it is the current cell
    // as well.
    assert_eq!(
        state_between((1, 2), (2, 2)),
        SolverCellState::Path,
        "the passage between `(1, 2)` and the goal"
    );
}

/// Item 1 again, on the run that reached the goal, where the cells that match
/// several conditions at once match the most.
///
/// The goal is the cell the last step expanded and the last cell of the path,
/// so it matches four conditions and shows the first of them.
#[test]
fn the_solver_walk_prefers_a_position_to_the_path_and_the_path_to_expanded() {
    let mut maze = hand_carved();
    let run = bfs_solved(&mut maze);
    let state = |x: u16, y: u16| {
        let (dx, dy) = at(x, y);
        run.solver_state(&maze, dx, dy)
    };

    assert_eq!(
        state(2, 2),
        SolverCellState::Goal,
        "the goal, and current, and on the path, and expanded"
    );
    assert_eq!(
        state(0, 0),
        SolverCellState::Start,
        "the start, and on the path, and expanded"
    );
    assert_eq!(
        state(0, 2),
        SolverCellState::Path,
        "on the path, and expanded"
    );
    assert_eq!(
        state(2, 1),
        SolverCellState::Expanded,
        "expanded, and nothing else"
    );
}

/// A Randomized Prim run over a fresh maze, advanced `steps` times.
fn prim_after(maze: &mut Maze, seed: u64, steps: u32) -> Run {
    let mut rng = rng::from_seed(seed);
    let mut run = Run::generating(maze, generator_named("prim"), &mut rng);
    advance(&mut run, maze, steps, "Randomized Prim");
    run
}

/// Every position of the display grid, in row-major order.
fn positions(maze: &Maze) -> Vec<(u16, u16)> {
    (0..maze.display_height())
        .flat_map(|dy| (0..maze.display_width()).map(move |dx| (dx, dy)))
        .collect()
}

/// Item 1 for generation: the walk over the four generation states.
///
/// The frontier of Randomized Prim is the set of uncarved cells beside the
/// carved region, so the maze alone says which cells are in it. That is what
/// the expectation below is built from, and it is why this test can name a
/// state without asking the generator for one.
#[test]
fn the_generation_walk_takes_the_first_condition_a_cell_matches() {
    let mut maze = Maze::new(5, 4);
    let run = prim_after(&mut maze, 3, 6);

    let mut currents = 0;
    for y in 0..maze.height() {
        for x in 0..maze.width() {
            let c = Cell { x, y };
            let (dx, dy) = at(x, y);
            let state = run.gen_state(&maze, dx, dy);
            if state == GenCellState::Current {
                currents += 1;
                assert!(maze.is_carved(c), "the current cell {c:?} is not carved");
                continue;
            }
            let beside_the_carved_region = Dir::ALL
                .into_iter()
                .filter_map(|d| maze.neighbour(c, d))
                .any(|n| maze.is_carved(n));
            let expected = if maze.is_carved(c) {
                GenCellState::Carved
            } else if beside_the_carved_region {
                GenCellState::Frontier
            } else {
                GenCellState::Uncarved
            };
            assert_eq!(state, expected, "at {c:?}");
        }
    }
    assert_eq!(currents, 1, "a run acts on one cell in the step now shown");
}

/// A wall of a part-carved maze is `Uncarved`, and so is every post.
#[test]
fn every_wall_position_of_a_generation_run_is_uncarved() {
    let mut maze = Maze::new(5, 4);
    let run = prim_after(&mut maze, 3, 6);

    for (dx, dy) in positions(&maze) {
        if maze.display_cell(dx, dy) == DisplayCell::Wall {
            assert_eq!(
                run.gen_state(&maze, dx, dy),
                GenCellState::Uncarved,
                "at ({dx}, {dy})"
            );
        }
    }
}

/// Item 2 for generation: a passage takes the weaker of its two ends here too.
///
/// The two generators differ in what that gives, because they differ in what
/// their frontier holds. Every passage of a Randomized Prim run joins two
/// carved cells and neither is in its frontier, so every passage is `Carved`,
/// which is what section 7.3 states. The frontier of the Recursive
/// Backtracker is its stack, and a stack cell is carved, so a passage between
/// two stack cells is `Frontier`. That is what makes the corridor the run is
/// walking read as one line, which is the reason section 7.3 gives for the
/// rule.
#[test]
fn a_generation_passage_takes_the_weaker_of_its_two_ends() {
    let mut maze = Maze::new(5, 4);
    let run = prim_after(&mut maze, 3, 6);
    for (dx, dy) in positions(&maze) {
        if matches!(maze.display_cell(dx, dy), DisplayCell::Passage(..)) {
            assert_eq!(
                run.gen_state(&maze, dx, dy),
                GenCellState::Carved,
                "every passage of a Randomized Prim run, and this one is at ({dx}, {dy})"
            );
        }
    }

    // One step of the Recursive Backtracker carves one edge out of `(0, 0)`,
    // and leaves both of its ends on the stack.
    let mut maze = Maze::new(5, 4);
    let mut rng = rng::from_seed(3);
    let mut run = Run::generating(&maze, generator_named("backtracker"), &mut rng);
    advance(&mut run, &mut maze, 1, "the Recursive Backtracker");

    let passages: Vec<(u16, u16)> = positions(&maze)
        .into_iter()
        .filter(|&(dx, dy)| matches!(maze.display_cell(dx, dy), DisplayCell::Passage(..)))
        .collect();
    assert_eq!(passages.len(), 1, "one step carves one edge");
    let (dx, dy) = passages[0];
    assert_eq!(
        run.gen_state(&maze, dx, dy),
        GenCellState::Frontier,
        "the passage between two cells on the stack"
    );
    let (dx, dy) = at(0, 0);
    assert_eq!(
        run.gen_state(&maze, dx, dy),
        GenCellState::Frontier,
        "`(0, 0)` is on the stack, below the cell the step acted on"
    );
}

/// Section 8.1 for a solver run: a step is one call to `step`, **expanded** is
/// a cell the solver took out of its frontier and processed, frontier is the
/// live size of the working set, and path length counts the cells from start
/// to goal inclusive.
#[test]
fn the_statistics_of_a_solver_run_count_steps_expanded_frontier_and_the_path() {
    let mut maze = hand_carved();
    let run = bfs_after(&mut maze, 3);
    let stats = run.stats(&maze);
    assert_eq!(stats.algorithm_short, "BFS", "the abbreviated name");
    assert_eq!(stats.steps, 3);
    assert_eq!(stats.expanded_or_carved, 3, "`(0,0)`, `(1,0)` and `(0,1)`");
    assert_eq!(stats.frontier, 3, "`(2,0)`, `(1,1)` and `(0,2)`");
    assert_eq!(stats.path_len, None, "there is no path before the goal");

    let mut maze = hand_carved();
    let run = bfs_solved(&mut maze);
    let stats = run.stats(&maze);
    assert_eq!(stats.steps, 9);
    assert_eq!(stats.expanded_or_carved, 9, "the nine cells of the maze");
    assert_eq!(stats.frontier, 0, "the queue is empty at the goal");
    assert_eq!(
        stats.path_len,
        Some(5),
        "`(0,0)`, `(0,1)`, `(0,2)`, `(1,2)` and the goal"
    );
}

/// Section 8.1 for a generation run: the **carved count** is the generation
/// counterpart of expanded, and a generation run has no path length.
#[test]
fn the_statistics_of_a_generation_run_count_carved_cells_and_no_path() {
    let mut maze = Maze::new(5, 4);
    let run = prim_after(&mut maze, 3, 6);
    let stats = run.stats(&maze);

    assert_eq!(stats.algorithm_short, "RandPrim", "the abbreviated name");
    assert_eq!(stats.steps, 6);
    assert_eq!(
        stats.expanded_or_carved, 7,
        "every step of Randomized Prim carves one cell, after the cell the run starts from"
    );
    assert_eq!(stats.path_len, None, "a generation run has no path");

    // The frontier the band draws is the frontier the screen draws. The cells
    // in the working set of Randomized Prim are uncarved, so none of them is
    // the cell the step acted on and each one draws as `Frontier`.
    let drawn = positions(&maze)
        .into_iter()
        .filter(|&(dx, dy)| run.gen_state(&maze, dx, dy) == GenCellState::Frontier)
        .count();
    assert_eq!(stats.frontier, drawn, "the count beside the screen");
}
