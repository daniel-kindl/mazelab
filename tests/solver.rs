//! Tier 2 of the test plan: solver correctness.
//!
//! The seven items of section 11.2 all land here. A solver reads a finished
//! maze, so every test below generates one first, and the generator it uses is
//! part of what fixes the maze it searches.
//!
//! The tests read a solver through its trait only. `is_expanded` and
//! `is_frontier` are the two questions the renderer asks, so a test that
//! reached inside a solver to count its working set would prove nothing about
//! what the screen shows.

use mazelab::StepOutcome;
use mazelab::braid::braid;
use mazelab::generator::{GENERATORS, GeneratorEntry};
use mazelab::maze::{Cell, Dir, Maze};
use mazelab::rng;
use mazelab::solver::{SOLVERS, Solver, SolverEntry};

/// The sizes every solver is run at: the smallest maze section 13 allows, a
/// non-square maze, and the 29 x 9 default of section 6.6.
const SIZES: [(u16, u16); 3] = [(4, 4), (7, 5), (29, 9)];

/// The seeds every solver is run at. Item 3 asks for a set of seeds, and the
/// same set carries the other items.
const SEEDS: [u64; 4] = [1, 7, 11, 2026];

/// The generator registry entry with this key.
fn generator_named(key: &str) -> &'static GeneratorEntry {
    GENERATORS
        .iter()
        .find(|e| e.key == key)
        .unwrap_or_else(|| panic!("no generator answers to {key}"))
}

/// A generated maze, made with the generator this key names.
fn generate(key: &str, width: u16, height: u16, seed: u64) -> Maze {
    let entry = generator_named(key);
    let mut maze = Maze::new(width, height);
    let mut rng = rng::from_seed(seed);
    let mut generator = (entry.make)(&maze, &mut rng);
    let cap = 8 * usize::from(width) * usize::from(height);
    for _ in 0..cap {
        if generator.step(&mut maze) == StepOutcome::Done {
            return maze;
        }
    }
    panic!("{key} did not finish inside {cap} steps")
}

/// Every cell of the maze, in row-major order. The step-by-step tests ask a
/// solver about each one, which is the only way to count a working set from
/// outside: rule 5 of the contract answers membership, and nothing iterates a
/// frontier.
fn cells(maze: &Maze) -> Vec<Cell> {
    (0..maze.height())
        .flat_map(|y| (0..maze.width()).map(move |x| Cell { x, y }))
        .collect()
}

/// The solver registry entry with this key.
fn solver_named(key: &str) -> &'static SolverEntry {
    SOLVERS
        .iter()
        .find(|e| e.key == key)
        .unwrap_or_else(|| panic!("no solver answers to {key}"))
}

/// The cell every run starts from, which is the corner the generators carve
/// from.
const START: Cell = Cell { x: 0, y: 0 };

/// The cell every run searches for, which is the far corner.
const fn goal_of(maze: &Maze) -> Cell {
    Cell {
        x: maze.width() - 1,
        y: maze.height() - 1,
    }
}

/// Runs a solver to `Done` and answers the solver, so the caller can ask it
/// for its path and its counts.
///
/// The step cap keeps a solver that never finishes from hanging the test run.
/// A solver expands each cell once and section 4.6 lets A\* discard a stale
/// entry as well, so `8 * W * H` is above any run these tests make and low
/// enough to fail in a moment.
///
/// Rule 3 of the contract is checked here, on every step of every run: the
/// path is `None` before `Done` and `Some` from `Done` onward.
fn run_to_done(entry: &SolverEntry, maze: &Maze) -> Box<dyn Solver> {
    let cap = 8 * usize::from(maze.width()) * usize::from(maze.height());
    let mut solver = (entry.make)(maze, START, goal_of(maze));
    for _ in 0..cap {
        assert!(
            solver.path().is_none(),
            "{} held a path before it reached the goal",
            entry.key
        );
        if solver.step(maze) == StepOutcome::Done {
            assert!(
                solver.path().is_some(),
                "{} returned Done without a path",
                entry.key
            );
            return solver;
        }
    }
    panic!("{} did not finish inside {cap} steps", entry.key)
}

/// The path a solver returns, as an owned value.
fn path_of(entry: &SolverEntry, maze: &Maze) -> Vec<Cell> {
    let solver = run_to_done(entry, maze);
    let path = solver
        .path()
        .unwrap_or_else(|| panic!("{} has no path", entry.key));
    path.to_vec()
}

/// The direction that crosses from `a` to `b`, or `None` when the two cells
/// are not neighbours.
fn direction(maze: &Maze, a: Cell, b: Cell) -> Option<Dir> {
    Dir::ALL
        .into_iter()
        .find(|&d| maze.neighbour(a, d) == Some(b))
}

/// Item 1 of section 11.2, as a check a test can make on any path: two cells
/// that follow each other are neighbours, and the wall between them is carved.
fn assert_walkable(maze: &Maze, path: &[Cell], key: &str) {
    for pair in path.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let d = direction(maze, a, b)
            .unwrap_or_else(|| panic!("{key} put {a:?} beside {b:?}, and they do not touch"));
        assert!(
            maze.is_open(a, d),
            "{key} walked from {a:?} to {b:?} through a wall that stands"
        );
    }
}

/// The two mazes the step-by-step tests walk: a small perfect maze, and the
/// braided 29 x 9 maze that `astar_discards_a_stale_entry_on_a_braided_maze`
/// below takes.
///
/// The braided one is not decoration. A **perfect** maze holds one path
/// between two cells, so no solver ever reaches a cell twice and the working
/// set holds no duplicate. Only the braided maze puts A\* through the lazy
/// deletion of section 4.6, which is where a count can drift.
fn mazes_to_search() -> Vec<Maze> {
    let mut braided = generate("prim", 29, 9, 7);
    braid(&mut braided, 0.5, &mut rng::from_seed(7));
    vec![generate("backtracker", 7, 5, 7), braided]
}

/// Item 1 of section 11.2: the path is walkable. Two cells that follow each
/// other in it are neighbours, and the wall between them is carved.
///
/// This is the test that separates a path from a list of cells: a solver that
/// reconstructed through a parent it never set, or that walked the parent
/// chain the wrong way, fails here.
#[test]
fn every_solver_returns_a_walkable_path() {
    for entry in SOLVERS {
        for (w, h) in SIZES {
            for seed in SEEDS {
                let maze = generate("backtracker", w, h, seed);
                let path = path_of(entry, &maze);
                assert_walkable(&maze, &path, entry.key);
            }
        }
    }
}

/// Item 2 of section 11.2: the path starts at the start and ends at the goal,
/// and it holds both. That is what **path length** counts.
#[test]
fn every_path_starts_at_the_start_and_ends_at_the_goal() {
    for entry in SOLVERS {
        for (w, h) in SIZES {
            for seed in SEEDS {
                let maze = generate("backtracker", w, h, seed);
                let path = path_of(entry, &maze);
                assert_eq!(
                    path.first().copied(),
                    Some(START),
                    "{} did not start at the start",
                    entry.key
                );
                assert_eq!(
                    path.last().copied(),
                    Some(goal_of(&maze)),
                    "{} did not end at the goal",
                    entry.key
                );
            }
        }
    }
}

/// Item 3 of section 11.2: BFS and A\* agree on path length, over a set of
/// seeds and sizes.
///
/// This is the test the `(f, h, insertion)` tie-break of section 4.6 must not
/// break. Taking the node closest to the goal first changes the *order of
/// expansion*, never the *length of the result*, because Manhattan distance is
/// admissible. A failure here is a wrong heuristic or a wrong `Ord`, not a
/// wrong tie-break.
///
/// A perfect maze holds exactly one simple path between two cells, so both
/// generators are run: on an unbraided maze the agreement is not the
/// interesting half. The braided maze of item 5 is where a second, longer path
/// exists for A\* to prefer by mistake.
#[test]
fn bfs_and_astar_agree_on_path_length() {
    for key in ["backtracker", "prim"] {
        for (w, h) in SIZES {
            for seed in SEEDS {
                for factor in [0.0, 0.5] {
                    let mut maze = generate(key, w, h, seed);
                    braid(&mut maze, factor, &mut rng::from_seed(seed));

                    let bfs = path_of(solver_named("bfs"), &maze);
                    let astar = path_of(solver_named("astar"), &maze);

                    assert_eq!(
                        bfs.len(),
                        astar.len(),
                        "BFS and A* disagree on {key} at {w} x {h}, seed {seed}, \
                         braid {factor}"
                    );
                }
            }
        }
    }
}

/// Item 4 of section 11.2: DFS finds a path, and its length is greater than or
/// equal to the length BFS finds. BFS is the one that proves the shortest, so
/// a DFS path below it would mean BFS is not shortest.
#[test]
fn dfs_finds_a_path_no_shorter_than_the_one_bfs_finds() {
    for (w, h) in SIZES {
        for seed in SEEDS {
            for factor in [0.0, 0.5] {
                let mut maze = generate("backtracker", w, h, seed);
                braid(&mut maze, factor, &mut rng::from_seed(seed));

                let dfs = path_of(solver_named("dfs"), &maze);
                let bfs = path_of(solver_named("bfs"), &maze);

                assert!(
                    dfs.len() >= bfs.len(),
                    "DFS found {} cells and BFS found {} at {w} x {h}, seed {seed}",
                    dfs.len(),
                    bfs.len()
                );
            }
        }
    }
}

/// Item 5 of section 11.2: every solver reaches the goal on a braided maze
/// too. Braiding adds loops, so a solver that leaned on the single path of a
/// **perfect** maze is found here.
#[test]
fn every_solver_reaches_the_goal_on_a_braided_maze() {
    for entry in SOLVERS {
        for (w, h) in SIZES {
            for seed in SEEDS {
                let mut maze = generate("prim", w, h, seed);
                braid(&mut maze, 1.0, &mut rng::from_seed(seed));

                let path = path_of(entry, &maze);
                assert_eq!(
                    path.last().copied(),
                    Some(goal_of(&maze)),
                    "{} did not reach the goal of a braided maze",
                    entry.key
                );
                // Item 1 again, where it can fail: a braided maze holds more
                // than one path between two cells, so it is the only maze on
                // which A* reaches a cell a second time and writes a parent
                // over the one the path is walked back along.
                assert_walkable(&maze, &path, entry.key);
            }
        }
    }
}

/// Item 6 of section 11.2: `expanded_count` never exceeds `W*H`, and every
/// cell on the path is expanded.
///
/// The first half is rule 4 of the contract: a cell is expanded once, so a
/// solver that counted a stale duplicate as an expansion counts past the cell
/// count here. The second half holds the statistics band honest: the path is
/// made of cells the solver took out of its frontier and processed.
#[test]
fn expanded_count_stays_inside_the_maze_and_covers_the_path() {
    for entry in SOLVERS {
        for (w, h) in SIZES {
            for seed in SEEDS {
                let maze = generate("backtracker", w, h, seed);
                let count = usize::from(w) * usize::from(h);
                let solver = run_to_done(entry, &maze);

                assert!(
                    solver.expanded_count() <= count,
                    "{} expanded {} cells of {count} at {w} x {h}",
                    entry.key,
                    solver.expanded_count()
                );
                let path = solver
                    .path()
                    .unwrap_or_else(|| panic!("{} has no path", entry.key));
                for &c in path {
                    assert!(
                        solver.is_expanded(c),
                        "{} put {c:?} on its path without expanding it",
                        entry.key
                    );
                }
            }
        }
    }
}

/// Item 6 of section 11.2, the other half of the count: the flags and the
/// count describe one set of expanded cells, at every step of a run. The
/// renderer asks the flag for each cell it draws and the statistics band draws
/// the count, so the two disagreeing would put a state on the screen that the
/// count denies.
#[test]
fn the_expanded_flags_and_the_expanded_count_agree_at_every_step() {
    for entry in SOLVERS {
        for maze in mazes_to_search() {
            let mut solver = (entry.make)(&maze, START, goal_of(&maze));
            let all = cells(&maze);

            let mut steps = 0;
            loop {
                let flagged = all.iter().filter(|&&c| solver.is_expanded(c)).count();
                assert_eq!(
                    flagged,
                    solver.expanded_count(),
                    "{} disagrees with itself after {steps} steps",
                    entry.key
                );
                let outcome = solver.step(&maze);
                steps += 1;
                assert!(
                    solver.current().is_some(),
                    "{} has no current cell after {steps} steps",
                    entry.key
                );
                if outcome == StepOutcome::Done {
                    break;
                }
            }
        }
    }
}

/// Rule 5 of the contract: `is_frontier` and `frontier_len` describe one
/// working set, at every step of a run.
///
/// A\* is why this is a test and not an assumption: its working set holds
/// stale duplicates, so its count is the number of cells in the open set and
/// not the length of its heap.
#[test]
fn the_frontier_flags_and_the_frontier_count_agree_at_every_step() {
    for entry in SOLVERS {
        for maze in mazes_to_search() {
            let mut solver = (entry.make)(&maze, START, goal_of(&maze));
            let all = cells(&maze);

            let mut steps = 0;
            loop {
                let flagged = all.iter().filter(|&&c| solver.is_frontier(c)).count();
                assert_eq!(
                    flagged,
                    solver.frontier_len(),
                    "{} disagrees with itself after {steps} steps",
                    entry.key
                );
                if solver.step(&maze) == StepOutcome::Done {
                    break;
                }
                steps += 1;
            }
        }
    }
}

/// Item 7 of section 11.2: a solver never changes the maze.
///
/// `Solver::step` takes `&Maze`, so the compiler carries this. The test is
/// here to catch interior mutability, which is the one way the signature can
/// be kept while the maze changes underneath. The whole display grid is
/// compared, which is every wall of the maze.
#[test]
fn no_solver_changes_the_maze() {
    for entry in SOLVERS {
        for (w, h) in SIZES {
            let maze = generate("backtracker", w, h, 7);
            let before = maze.display_grid();
            let carved_before = maze.carved_count();

            run_to_done(entry, &maze);

            assert_eq!(
                maze.display_grid(),
                before,
                "{} changed a wall at {w} x {h}",
                entry.key
            );
            assert_eq!(maze.carved_count(), carved_before, "{}", entry.key);
        }
    }
}

/// Section 4.6 holds one open set and one heap that is allowed to disagree
/// with it, and this is the test that the disagreement is handled.
///
/// A\* pushes a cell a second time when it reaches it by a shorter path,
/// because a `BinaryHeap` has no decrease-key. The older entry stays in the
/// heap and is discarded when it pops. A run that takes more steps than it
/// expanded cells is a run that discarded one, so this maze proves the branch
/// runs; the counts around it are what the two agreement tests above check on
/// the same maze.
///
/// The maze is pinned, because a **perfect** maze never produces a duplicate:
/// it holds one path between two cells. Braiding is what makes a second path,
/// and this is a size and a seed where the pass makes enough of them.
#[test]
fn astar_discards_a_stale_entry_on_a_braided_maze() {
    let mut maze = generate("prim", 29, 9, 7);
    braid(&mut maze, 0.5, &mut rng::from_seed(7));

    let entry = solver_named("astar");
    let mut solver = (entry.make)(&maze, START, goal_of(&maze));
    let mut steps = 0;
    while solver.step(&maze) != StepOutcome::Done {
        steps += 1;
    }
    steps += 1;

    assert!(
        steps > solver.expanded_count(),
        "A* took {steps} steps and expanded {} cells, so it discarded no stale entry",
        solver.expanded_count()
    );
}
