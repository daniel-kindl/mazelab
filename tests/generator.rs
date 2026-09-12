//! Tier 1 of the test plan, for the two generators and the braiding pass.
//!
//! The items of section 11.1 that land here are 1 (a stored reference vector),
//! 2 (a maze is connected), 3 (an unbraided maze has `W*H - 1` edges), 4 and 5
//! (the braiding pass) and 10 (both generators terminate and carve every
//! cell). The maze invariants of items 6 and 7 are in `tests/maze.rs`.
//!
//! The braiding pass is tested here rather than in a file of its own, because
//! every braid test needs a generated maze to braid.

use mazelab::StepOutcome;
use mazelab::braid::braid;
use mazelab::generator::{GENERATORS, GeneratorEntry};
use mazelab::maze::{Cell, Dir, DisplayCell, Maze};
use mazelab::rng;

/// The sizes every generator is run at: the smallest maze section 13 allows, a
/// non-square maze, and the 29 x 9 default of section 6.6.
const SIZES: [(u16, u16); 3] = [(4, 4), (7, 5), (29, 9)];

/// Every cell of the maze, in row-major order.
fn cells(maze: &Maze) -> Vec<Cell> {
    let mut all = Vec::new();
    for y in 0..maze.height() {
        for x in 0..maze.width() {
            all.push(Cell { x, y });
        }
    }
    all
}

/// The offset of a cell in a flag grid of `W * H`.
fn index(maze: &Maze, c: Cell) -> usize {
    usize::from(c.y) * usize::from(maze.width()) + usize::from(c.x)
}

/// The neighbour of `c` in direction `d`, where the caller knows there is one.
fn neighbour_of(maze: &Maze, c: Cell, d: Dir) -> Cell {
    maze.neighbour(c, d)
        .unwrap_or_else(|| panic!("{c:?} has no neighbour towards {d:?}"))
}

/// Runs a generator to `Done` and answers the number of steps it took.
///
/// The step cap is what keeps a generator that never finishes from hanging the
/// test run. Section 4.1 puts the longer of the two runs at roughly `2 * W * H`
/// steps.
fn run_to_done(entry: &GeneratorEntry, maze: &mut Maze, seed: u64) -> usize {
    let cap = 8 * usize::from(maze.width()) * usize::from(maze.height());
    let mut rng = rng::from_seed(seed);
    let mut generator = (entry.make)(maze, &mut rng);
    for steps in 1..=cap {
        if generator.step(maze) == StepOutcome::Done {
            assert_eq!(generator.frontier_len(), 0, "{} left a frontier", entry.key);
            return steps;
        }
    }
    panic!("{} did not finish inside {cap} steps", entry.key)
}

/// A generated maze, with the generator's own steps thrown away.
fn generate(entry: &GeneratorEntry, width: u16, height: u16, seed: u64) -> Maze {
    let mut maze = Maze::new(width, height);
    run_to_done(entry, &mut maze, seed);
    maze
}

/// The number of cells a flood fill from `(0, 0)` reaches through carved edges.
fn reached_from_origin(maze: &Maze) -> usize {
    let mut seen = vec![false; usize::from(maze.width()) * usize::from(maze.height())];
    let mut stack = vec![Cell { x: 0, y: 0 }];
    seen[0] = true;
    let mut reached = 0;
    while let Some(c) = stack.pop() {
        reached += 1;
        for d in Dir::ALL {
            if maze.is_open(c, d) {
                let n = neighbour_of(maze, c, d);
                let i = index(maze, n);
                if !seen[i] {
                    seen[i] = true;
                    stack.push(n);
                }
            }
        }
    }
    reached
}

/// Every dead end of the maze, in row-major order.
fn dead_ends(maze: &Maze) -> Vec<Cell> {
    cells(maze)
        .into_iter()
        .filter(|&c| maze.degree(c) == 1)
        .collect()
}

/// Every carved edge of the maze, each named by a cell and a direction. One
/// edge appears twice, once from each of its two cells, which is what makes
/// this the right value to prove that a pass added edges and removed none.
fn open_walls(maze: &Maze) -> Vec<(Cell, Dir)> {
    let mut open = Vec::new();
    for c in cells(maze) {
        for d in Dir::ALL {
            if maze.is_open(c, d) {
                open.push((c, d));
            }
        }
    }
    open
}

/// The maze drawn as one string for each row of its display grid, with `#` for
/// a wall and a space for a cell or a passage. This is the form the stored
/// reference vectors of item 1 take: a picture a reader can check by eye.
fn picture(maze: &Maze) -> Vec<String> {
    (0..maze.display_height())
        .map(|dy| {
            (0..maze.display_width())
                .map(|dx| match maze.display_cell(dx, dy) {
                    DisplayCell::Wall => '#',
                    DisplayCell::Cell(_) | DisplayCell::Passage(_, _) => ' ',
                })
                .collect()
        })
        .collect()
}

/// The registry entry with this key.
fn entry_named(key: &str) -> &'static GeneratorEntry {
    GENERATORS
        .iter()
        .find(|e| e.key == key)
        .unwrap_or_else(|| panic!("no generator answers to {key}"))
}

/// Item 1 of section 11.1: seed 1 at 4 x 4 carves one fixed maze with the
/// Recursive Backtracker, committed here as a picture.
///
/// The vector is stored, not recomputed. Generating twice and comparing would
/// pass even after the RNG changed under the crate, and that is the failure
/// this test exists to catch.
#[test]
fn the_backtracker_at_seed_1_and_4x4_carves_a_known_maze() {
    let maze = generate(entry_named("backtracker"), 4, 4, 1);
    assert_eq!(
        picture(&maze),
        vec![
            "#########",
            "#   #   #",
            "### # ###",
            "#   #   #",
            "# ##### #",
            "# #   # #",
            "# # # # #",
            "#   #   #",
            "#########",
        ]
    );
}

/// Item 1 of section 11.1, for the second generator. The two pictures differ,
/// which is the other half of the claim: the generator is part of what fixes a
/// maze, beside the seed and the size.
#[test]
fn prim_at_seed_1_and_4x4_carves_a_known_maze() {
    let maze = generate(entry_named("prim"), 4, 4, 1);
    assert_eq!(
        picture(&maze),
        vec![
            "#########",
            "#   #   #",
            "# # ### #",
            "# #     #",
            "##### # #",
            "# #   # #",
            "# ### ###",
            "#       #",
            "#########",
        ]
    );
}

/// Item 1 of section 11.1, for the braiding pass: one fixed braided maze, at
/// seed 11 and 7 x 5 and factor 1.
///
/// Randomized Prim carries this one, because its maze is where the pass does
/// the most work. This one holds 11 dead ends and the pass clears all 11 with
/// 7 carves, so 4 of them are cleared as the partner of a carve, and the
/// `degree(c) != 1` re-check of section 4.3 skips those 4 when their own turn
/// comes. The size is chosen for the same reason: at 4 x 4 the pass pairs two
/// dead ends only twice, and a pass that drew its partner without preferring a
/// dead end still draws the same picture one time in four.
///
/// A pass that dropped the re-check, or the preference, or the row-major
/// collect before the shuffle, draws a different picture here.
#[test]
fn prim_at_seed_11_and_7x5_braided_at_1_carves_a_known_maze() {
    let mut maze = generate(entry_named("prim"), 7, 5, 11);
    let unbraided = maze.edge_count();
    assert_eq!(dead_ends(&maze).len(), 11);

    braid(&mut maze, 1.0, &mut rng::from_seed(11));

    assert_eq!(maze.edge_count(), unbraided + 7);
    assert_eq!(dead_ends(&maze).len(), 0);
    assert_eq!(
        picture(&maze),
        vec![
            "###############",
            "#             #",
            "# # ### ### # #",
            "# # #     #   #",
            "# # # ### #####",
            "# # #     #   #",
            "# # # ##### # #",
            "# #           #",
            "# # # ##### # #",
            "#   #         #",
            "###############",
        ]
    );
}

/// Item 2 of section 11.1: a flood fill from `(0, 0)` reaches every cell.
#[test]
fn every_generator_leaves_a_connected_maze() {
    for entry in GENERATORS {
        for (w, h) in SIZES {
            let maze = generate(entry, w, h, 7);
            assert_eq!(
                reached_from_origin(&maze),
                usize::from(w) * usize::from(h),
                "{} at {w} x {h} left a cell the flood fill cannot reach",
                entry.key
            );
        }
    }
}

/// Item 3 of section 11.1: an unbraided maze holds exactly `W*H - 1` carved
/// edges. With item 2 that is the spanning tree, which is what makes the maze
/// **perfect**.
#[test]
fn every_generator_leaves_exactly_one_edge_short_of_the_cell_count() {
    for entry in GENERATORS {
        for (w, h) in SIZES {
            let maze = generate(entry, w, h, 7);
            assert_eq!(
                maze.edge_count(),
                usize::from(w) * usize::from(h) - 1,
                "{} at {w} x {h} did not carve a spanning tree",
                entry.key
            );
        }
    }
}

/// Item 10 of section 11.1: both generators terminate and leave every cell
/// carved. The step counts are the ones sections 4.1 and 4.2 state: the
/// backtracker pushes and pops each cell, and Randomized Prim carves one cell
/// for each step after the cell it starts from.
#[test]
fn every_generator_terminates_and_carves_every_cell() {
    for (w, h) in SIZES {
        let count = usize::from(w) * usize::from(h);

        let mut maze = Maze::new(w, h);
        let steps = run_to_done(entry_named("backtracker"), &mut maze, 7);
        assert_eq!(maze.carved_count(), count, "the backtracker at {w} x {h}");
        assert_eq!(steps, 2 * count - 1, "the backtracker at {w} x {h}");

        let mut maze = Maze::new(w, h);
        let steps = run_to_done(entry_named("prim"), &mut maze, 7);
        assert_eq!(maze.carved_count(), count, "Randomized Prim at {w} x {h}");
        assert_eq!(steps, count - 1, "Randomized Prim at {w} x {h}");
    }
}

/// Rule 5 of the generator contract: `is_frontier` and `frontier_len` describe
/// one working set, at every step of a run. The renderer asks the first for
/// each cell it draws and the statistics band draws the second, so the two
/// disagreeing would put a frontier on the screen that the count denies.
#[test]
fn the_frontier_flags_and_the_frontier_count_agree_at_every_step() {
    for entry in GENERATORS {
        let mut maze = Maze::new(7, 5);
        let mut rng = rng::from_seed(7);
        let mut generator = (entry.make)(&maze, &mut rng);
        let all = cells(&maze);

        let mut steps = 0;
        loop {
            let flagged = all.iter().filter(|&&c| generator.is_frontier(c)).count();
            assert_eq!(
                flagged,
                generator.frontier_len(),
                "{} disagrees with itself after {steps} steps",
                entry.key
            );
            if generator.step(&mut maze) == StepOutcome::Done {
                break;
            }
            steps += 1;
            assert!(
                generator.current().is_some(),
                "{} has no current cell",
                entry.key
            );
        }
    }
}

/// Item 4 of section 11.1: braiding at factor 0 changes nothing. A maze that a
/// pass may not touch is compared by its whole display grid, which is every
/// wall of it.
#[test]
fn braiding_at_factor_0_changes_nothing() {
    for entry in GENERATORS {
        for (w, h) in SIZES {
            let mut maze = generate(entry, w, h, 3);
            let before = maze.display_grid();
            let carved_before = maze.carved_count();

            braid(&mut maze, 0.0, &mut rng::from_seed(3));

            assert_eq!(
                maze.display_grid(),
                before,
                "{} braided at 0 at {w} x {h}",
                entry.key
            );
            assert_eq!(maze.carved_count(), carved_before, "{}", entry.key);
        }
    }
}

/// Item 5 of section 11.1: braiding at factor `f` removes about `f` of the
/// dead ends.
///
/// The bound is a range, not a number, and section 4.3 gives the reason.
/// Braiding takes `f` of the dead ends and carves one edge out of each, so at
/// least that many stop being dead ends. It prefers a neighbour that is itself
/// a dead end, so one carve can remove two, and the re-check then skips a cell
/// a previous carve already opened. Twice the take is the ceiling.
#[test]
fn braiding_removes_about_the_factor_of_the_dead_ends() {
    for entry in GENERATORS {
        for (w, h) in SIZES {
            // The factor as a percentage beside it, so that the take is
            // derived here in whole numbers. Recomputing it with the `f64`
            // expression the pass itself uses would let a rounding error pass.
            for (factor, percent) in [(0.25, 25), (0.5, 50), (1.0, 100)] {
                let mut maze = generate(entry, w, h, 11);
                let before = dead_ends(&maze).len();
                let edges_before = maze.edge_count();
                // Rounded half up, which is what `f64::round` does for a
                // count. The three factors are all exact in binary, so the two
                // derivations cannot drift apart.
                let take = (before * percent + 50) / 100;

                braid(&mut maze, factor, &mut rng::from_seed(11));

                let after = dead_ends(&maze).len();
                assert!(
                    after <= before - take,
                    "{} at {factor} and {w} x {h} left {after} of {before} dead ends, \
                     and had to remove {take}",
                    entry.key
                );
                assert!(
                    after >= before.saturating_sub(2 * take),
                    "{} at {factor} and {w} x {h} removed more than two dead ends \
                     for each carve",
                    entry.key
                );
                // Every carve is a carve out of a dead end, and no two carves
                // start from the same cell, so each one removes at least the
                // cell it starts from. The pass can therefore never add more
                // edges than it removes dead ends.
                assert!(
                    maze.edge_count() - edges_before <= before - after,
                    "{} at {factor} and {w} x {h} added {} edges and removed {} dead ends",
                    entry.key,
                    maze.edge_count() - edges_before,
                    before - after
                );
            }
        }
    }
}

/// Item 5 of section 11.1: braiding only ever adds edges, and it never
/// disconnects the maze. It carves, and a carve cannot put a wall back, so a
/// wall that stands after the pass stood before it.
#[test]
fn braiding_only_adds_edges_and_disconnects_nothing() {
    for entry in GENERATORS {
        for (w, h) in SIZES {
            let mut maze = generate(entry, w, h, 11);
            let before = open_walls(&maze);
            let count = usize::from(w) * usize::from(h);

            braid(&mut maze, 1.0, &mut rng::from_seed(11));

            for (c, d) in before {
                assert!(
                    maze.is_open(c, d),
                    "{} braided the wall at {c:?} towards {d:?} shut at {w} x {h}",
                    entry.key
                );
            }
            assert!(
                maze.edge_count() >= count - 1,
                "{} lost edges at {w} x {h}",
                entry.key
            );
            assert_eq!(
                reached_from_origin(&maze),
                count,
                "{} disconnected the maze at {w} x {h}",
                entry.key
            );
        }
    }
}

/// A braided maze holds loops, so it is no longer **perfect**. This is the
/// other side of item 4: a factor of 0 must change nothing, and a factor above
/// 0 must change something.
#[test]
fn braiding_above_0_adds_at_least_one_loop() {
    for entry in GENERATORS {
        for (w, h) in SIZES {
            let mut maze = generate(entry, w, h, 11);
            let spanning_tree = maze.edge_count();

            braid(&mut maze, 0.5, &mut rng::from_seed(11));

            assert!(
                maze.edge_count() > spanning_tree,
                "{} braided at 0.5 and added no edge at {w} x {h}",
                entry.key
            );
        }
    }
}
