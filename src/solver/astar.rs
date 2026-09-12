//! The A* solver.
//!
//! Section 4.6 of `SPEC.md` holds the algorithm. The **frontier** is the open
//! set, and A* differs from BFS in which cell of it the next step takes: the
//! one with the lowest `f`, where `f` is the steps already walked plus the
//! Manhattan distance still to walk.
//!
//! Manhattan distance is admissible here: a step between two adjacent cells
//! costs 1, and a wall can only make the true path longer than the
//! straight-line count. An admissible heuristic never sends A* past a shorter
//! path, which is why A* and BFS return paths of the same length.
//!
//! The open set is a [`BinaryHeap`], which has no decrease-key, so a cell that
//! is reached again by a shorter path is pushed a second time. The older entry
//! stays in the heap and is discarded when it pops, which is **lazy deletion**.
//! The flag grid beside the heap is what keeps the count of the frontier right
//! while the heap holds those duplicates.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::{Search, Solver, reachable};
use crate::StepOutcome;
use crate::maze::{Cell, Maze};

/// One entry of the open set.
///
/// `f` and `h` are held rather than recomputed, because the ordering is asked
/// for them on every heap operation and a cell alone cannot answer: `f` is a
/// property of the entry, and a later entry for the same cell holds a lower
/// one.
struct Node {
    /// The steps walked to the cell, plus the heuristic from it to the goal.
    f: u32,
    /// The heuristic from the cell to the goal, for the tie-break.
    h: u32,
    /// The order this entry was pushed in, for the second tie-break.
    insertion: u32,
    /// The cell this entry stands for.
    cell: Cell,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Inverted: BinaryHeap is a max-heap and the smallest (f, h, insertion)
        // must pop first.
        //
        // The `h` term is deliberate and it is not the standard tie-break.
        // Among nodes with equal `f`, taking the one closest to the goal first
        // drives the search forward in a visible spike. Breaking on `f` alone
        // lets A* expand a broad ring, which on a maze looks exactly like BFS,
        // and MazeLab exists to show the difference between the two.
        //
        // `insertion` keeps the order stable among nodes with equal `f` and
        // equal `h`, so a run is reproducible. Do not "correct" this to a
        // tie-break on `f` alone without reading section 4.6 of SPEC.md.
        other
            .f
            .cmp(&self.f)
            .then_with(|| other.h.cmp(&self.h))
            .then_with(|| other.insertion.cmp(&self.insertion))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Node {}

/// Equality over the three fields the ordering compares, and no other. The
/// `Ord` contract asks for a total order that agrees with equality, and
/// `insertion` is unique, so no two entries are ever equal.
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.h == other.h && self.insertion == other.insertion
    }
}

/// The Manhattan distance from `c` to `goal`, which is the heuristic.
fn h(c: Cell, goal: Cell) -> u32 {
    u32::from(c.x.abs_diff(goal.x)) + u32::from(c.y.abs_diff(goal.y))
}

/// The A* solver, carrying its open set.
struct Astar {
    /// The open set. It holds a stale entry for each cell that was reached
    /// again by a shorter path, so its length is not the size of the frontier.
    open: BinaryHeap<Node>,
    /// One flag for each cell: the cell is in the open set. Rule 5 of the
    /// contract asks for an O(1) `is_frontier`, and this is what answers it,
    /// once for a cell the heap holds several entries for.
    in_open: Vec<bool>,
    /// The number of set flags in `in_open`, which is the size of the
    /// **frontier** the statistics band draws.
    in_open_count: usize,
    /// The steps walked to each cell on the best path found so far.
    /// `u32::MAX` for a cell no step has reached.
    g: Vec<u32>,
    /// The order the next entry is pushed in. Entry 0 is the start.
    next_insertion: u32,
    /// The expanded flags, the parents and the path.
    search: Search,
}

/// Makes the A* solver, which searches `maze` from `start` for `goal`.
///
/// The open set holds the start cell, so the first [`Solver::step`] expands it.
#[must_use]
pub fn make(maze: &Maze, start: Cell, goal: Cell) -> Box<dyn Solver> {
    let search = Search::new(maze, start, goal);
    let cells = search.cell_count();
    let mut in_open = vec![false; cells];
    let mut g = vec![u32::MAX; cells];
    in_open[search.index(start)] = true;
    g[search.index(start)] = 0;

    let mut open = BinaryHeap::new();
    open.push(Node {
        f: h(start, goal),
        h: h(start, goal),
        insertion: 0,
        cell: start,
    });

    Box::new(Astar {
        open,
        in_open,
        in_open_count: 1,
        g,
        next_insertion: 1,
        search,
    })
}

impl Solver for Astar {
    fn step(&mut self, maze: &Maze) -> StepOutcome {
        // The caller does not step after `Done`, so the heap holds an entry.
        let Some(node) = self.open.pop() else {
            // Unreachable, for the reason the same arm of `dfs` gives: the
            // arm answers `Done` without a path, and a connected maze never
            // takes a solver there.
            debug_assert!(false, "A* ran out of cells before it found the goal");
            return StepOutcome::Done;
        };
        let c = node.cell;

        if self.search.is_expanded(c) {
            // Lazy deletion of a stale entry: a better entry for this cell
            // popped before it and took the cell out of the open set then.
            // Discarding an entry is a step too, which is rule 1.
            //
            // This check stands before the two lines below it, and it must
            // stay there. A stale pop that lowered the count would lower it
            // once for every duplicate the heap still holds, and the count
            // would go below zero. Only the pop that expands a cell takes that
            // cell out of the open set, which is what section 4.6 asks for
            // with `is_frontier(c) = in_open[c] && !expanded[c]` beside
            // `frontier_len() = in_open_count`.
            return StepOutcome::Stepped;
        }
        let i = self.search.index(c);
        self.in_open[i] = false;
        self.in_open_count -= 1;

        self.search.expand(c);
        if c == self.search.goal() {
            return self.search.finish();
        }

        // The neighbours through a carved edge, in the fixed N E S W order.
        // Every edge costs one step, so the cost to a neighbour is the cost to
        // `c` plus one.
        let goal = self.search.goal();
        for n in reachable(maze, c) {
            let j = self.search.index(n);
            let tentative = self.g[i] + 1;
            if tentative < self.g[j] {
                self.g[j] = tentative;
                self.search.set_parent(n, c);
                self.open.push(Node {
                    f: tentative + h(n, goal),
                    h: h(n, goal),
                    insertion: self.next_insertion,
                    cell: n,
                });
                self.next_insertion += 1;
                if !self.in_open[j] {
                    self.in_open[j] = true;
                    self.in_open_count += 1;
                }
            }
        }
        StepOutcome::Stepped
    }

    fn current(&self) -> Option<Cell> {
        self.search.current()
    }

    fn is_frontier(&self, c: Cell) -> bool {
        // A cell leaves the open set on the pop that expands it, so the second
        // half only answers a cell the heap holds a stale entry for.
        !self.search.is_expanded(c) && self.search.offset(c).is_some_and(|i| self.in_open[i])
    }

    fn frontier_len(&self) -> usize {
        self.in_open_count
    }

    fn is_expanded(&self, c: Cell) -> bool {
        self.search.is_expanded(c)
    }

    fn expanded_count(&self) -> usize {
        self.search.expanded_count()
    }

    fn path(&self) -> Option<&[Cell]> {
        self.search.path()
    }
}
