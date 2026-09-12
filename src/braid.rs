//! The braiding pass, which removes a fraction of the dead ends.
//!
//! Section 4.3 of `SPEC.md` holds the pass and ADR 0004 holds the reason it is
//! separate: a generator leaves a **perfect** maze, and every loop in a maze
//! comes from here. The pass is a free function and not a third trait, because
//! it is instant. It is never animated, so it needs no working set to draw and
//! no step to advance.
//!
//! The tier-1 tests that cover the pass are items 1, 4 and 5, and they are in
//! `tests/generator.rs`, beside the generators whose mazes they braid.

use rand::RngExt as _;
use rand::seq::SliceRandom as _;

use crate::maze::{Cell, Dir, Maze};
use crate::rng::Rng;

/// Removes `factor` of the dead ends of the maze, by carving one more edge out
/// of each dead end it takes.
///
/// A **braid factor** of 0 changes nothing and keeps the maze **perfect**. A
/// factor above 0 adds loops, so more than one path can connect two cells. The
/// pass only ever carves, so it can never disconnect a maze and never puts a
/// wall back.
///
/// It draws from `rng`, which is the RNG generation was started from. A
/// generator runs on a child of that RNG, taken with
/// [`rng::split`](crate::rng::split), so the pass never draws a number
/// generation drew, and the seed still reproduces the braided maze. ADR 0005
/// holds the rule.
pub fn braid(maze: &mut Maze, factor: f64, rng: &mut Rng) {
    if factor <= 0.0 {
        return;
    }

    // Row-major, and collected before the shuffle. That is what makes the pass
    // reproducible: the order of a collection must never come from a hash.
    let mut ends = dead_ends(maze);
    ends.shuffle(rng);

    // A dead end is a cell, and a cell count always fits a `u32`: a maze is
    // `u16 x u16` and `u16::MAX * u16::MAX` is below `u32::MAX`.
    let Ok(count) = u32::try_from(ends.len()) else {
        unreachable!("a maze cannot hold {} dead ends", ends.len())
    };

    // How many dead ends the pass takes. The take stays an `f64`, because the
    // factor is an `f64`: turning it into an integer here would need a cast out
    // of the arithmetic that defines it. Every count in this pass is far inside
    // the integers an `f64` holds exactly. A factor above 1 takes the whole
    // list, because the counter runs out first.
    let take = (f64::from(count) * factor).round();

    for (taken, &c) in (0u32..).zip(&ends) {
        if f64::from(taken) >= take {
            break;
        }
        // An earlier carve in this pass may have opened this dead end already,
        // and a cell that is no longer a dead end must not be carved twice.
        if maze.degree(c) != 1 {
            continue;
        }
        if let Some(n) = partner(maze, c, rng) {
            maze.carve(c, n);
        }
    }
}

/// Every cell with exactly one carved edge, in row-major order.
fn dead_ends(maze: &Maze) -> Vec<Cell> {
    let mut ends = Vec::new();
    for y in 0..maze.height() {
        for x in 0..maze.width() {
            let c = Cell { x, y };
            if maze.degree(c) == 1 {
                ends.push(c);
            }
        }
    }
    ends
}

/// The neighbour of `c` that the pass carves to, or `None` when `c` has no
/// wall left that stands between it and a neighbour. Only a maze one cell wide
/// can hold such a dead end, and section 13 puts the floor of both axes at 4.
///
/// The candidates are the neighbours whose wall still stands, in the fixed
/// `N, E, S, W` order. A neighbour that is itself a dead end is preferred,
/// because that carve removes two dead ends instead of one, and that is what
/// makes a low braid factor visible.
fn partner(maze: &Maze, c: Cell, rng: &mut Rng) -> Option<Cell> {
    // Both lists are filled with `c`, which is never its own neighbour, and
    // only the first entries of each are ever read.
    let mut walled = [c; 4];
    let mut walled_count: u32 = 0;
    let mut ends = [c; 4];
    let mut ends_count: u32 = 0;

    for d in Dir::ALL {
        if let Some(n) = maze.neighbour(c, d)
            && !maze.is_open(c, d)
        {
            walled[walled_count as usize] = n;
            walled_count += 1;
            if maze.degree(n) == 1 {
                ends[ends_count as usize] = n;
                ends_count += 1;
            }
        }
    }

    let (pool, count) = if ends_count > 0 {
        (ends, ends_count)
    } else {
        (walled, walled_count)
    };
    if count == 0 {
        return None;
    }
    Some(pool[rng.random_range(0u32..count) as usize])
}
