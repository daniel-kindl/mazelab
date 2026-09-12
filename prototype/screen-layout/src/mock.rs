//! Hard-coded MazeLab state. THROWAWAY.
//!
//! Nothing here is an algorithm. The maze and both run snapshots were produced
//! offline (seed 445, Recursive Backtracker, then A* and BFS) and pasted in as
//! literals, so that the mock renders a realistic picture without carrying any
//! of the code that issue #6 says not to write.

pub const SEED: u64 = 445;
pub const W: usize = 11;
pub const H: usize = 6;

/// The finished maze, as the (2W+1) x (2H+1) display grid. '#' is a wall
/// position, ' ' is an open position.
pub const SOLVED_GRID: [&str; 2 * H + 1] = [
    "#######################",
    "#   #     #       #   #",
    "### # # ### # ### # # #",
    "# # # #   # # #     # #",
    "# # ##### # # ####### #",
    "# #     #   #   #   # #",
    "# ##### ####### ### # #",
    "#     #       #   # # #",
    "# ### ####### ### # # #",
    "# #         # #   #   #",
    "# ########### # ### ###",
    "#               #     #",
    "#######################",
];

/// The same maze 45 generator steps in: 29 of 66 cells carved.
pub const GEN_GRID: [&str; 2 * H + 1] = [
    "#######################",
    "#   ###################",
    "### ###################",
    "# # ###################",
    "# # ###################",
    "# #     ###############",
    "# ##### ###############",
    "#     #       #########",
    "# ### ####### #########",
    "# #         # #########",
    "# ########### #########",
    "#             #########",
    "#######################",
];

pub type Pos = (usize, usize);

pub const START: Pos = (0, 0);
pub const GOAL: Pos = (10, 5);
/// The cell the solver is acting on in the step now shown. It sits on the
/// final path on purpose, so the priority order of decision 17 is visible.
pub const CURRENT: Pos = (9, 5);

pub const PATH: &[Pos] = &[
    (0, 0), (1, 0), (1, 1), (1, 2), (2, 2), (3, 2), (3, 3), (4, 3), (5, 3),
    (6, 3), (6, 4), (6, 5), (7, 5), (7, 4), (8, 4), (8, 3), (7, 3), (7, 2),
    (6, 2), (6, 1), (6, 0), (7, 0), (8, 0), (8, 1), (9, 1), (9, 0), (10, 0),
    (10, 1), (10, 2), (10, 3), (10, 4), (9, 4), (9, 5), (10, 5),
];

pub const EXPANDED: &[Pos] = &[
    (0, 0), (0, 3), (0, 4), (0, 5), (1, 0), (1, 1), (1, 2), (1, 3), (1, 5),
    (2, 2), (2, 3), (2, 4), (2, 5), (3, 2), (3, 3), (3, 4), (3, 5), (4, 3),
    (4, 4), (4, 5), (5, 0), (5, 1), (5, 2), (5, 3), (5, 4), (5, 5), (6, 0),
    (6, 1), (6, 2), (6, 3), (6, 4), (6, 5), (7, 0), (7, 1), (7, 2), (7, 3),
    (7, 4), (7, 5), (8, 0), (8, 1), (8, 3), (8, 4), (9, 0), (9, 1), (9, 4),
    (9, 5), (10, 0), (10, 1), (10, 2), (10, 3), (10, 4), (10, 5),
];

pub const FRONTIER: &[Pos] = &[(0, 2), (1, 4), (4, 2), (8, 5), (9, 3)];

pub const GEN_CURRENT: Pos = (6, 5);

pub const GEN_CARVED: &[Pos] = &[
    (0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (1, 0), (1, 1), (1, 2),
    (1, 3), (1, 4), (1, 5), (2, 2), (2, 3), (2, 4), (2, 5), (3, 2), (3, 3),
    (3, 4), (3, 5), (4, 3), (4, 4), (4, 5), (5, 3), (5, 4), (5, 5), (6, 3),
    (6, 4), (6, 5),
];

/// The Recursive Backtracker's stack is its frontier (decision 17).
pub const GEN_STACK: &[Pos] = &[
    (0, 0), (1, 0), (1, 1), (1, 2), (2, 2), (3, 2), (3, 3), (4, 3), (5, 3),
    (6, 3), (6, 4), (6, 5),
];

// ---------------------------------------------------------------- cell state

/// Decision 17, solver order: start > goal > current > path > frontier >
/// expanded > open floor > wall.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Start,
    Goal,
    Current,
    Path,
    Frontier,
    Expanded,
    Floor,
    Wall,
}

/// Decision 17, generation order: current > frontier > carved > uncarved.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GenCell {
    Current,
    Frontier,
    Carved,
    Uncarved,
}

fn has(set: &[Pos], p: Pos) -> bool {
    set.contains(&p)
}

fn path_step(a: Pos, b: Pos) -> bool {
    PATH.windows(2).any(|w| (w[0] == a && w[1] == b) || (w[0] == b && w[1] == a))
}

fn cell_state(p: Pos) -> Cell {
    if p == START {
        Cell::Start
    } else if p == GOAL {
        Cell::Goal
    } else if p == CURRENT {
        Cell::Current
    } else if has(PATH, p) {
        Cell::Path
    } else if has(FRONTIER, p) {
        Cell::Frontier
    } else if has(EXPANDED, p) {
        Cell::Expanded
    } else {
        Cell::Floor
    }
}

/// Resolve one display-grid position to a solver cell state.
///
/// Odd/odd positions are cells. The rest are wall positions; an open one is a
/// carved edge, and it inherits the state of the edge it joins.
pub fn solver_at(x: usize, y: usize) -> Cell {
    let ch = SOLVED_GRID[y].as_bytes()[x];
    if ch == b'#' {
        return Cell::Wall;
    }
    if x % 2 == 1 && y % 2 == 1 {
        return cell_state((x / 2, y / 2));
    }
    // An open edge between two cells.
    let (a, b) = if x % 2 == 0 {
        ((x / 2 - 1, y / 2), (x / 2, y / 2))
    } else {
        ((x / 2, y / 2 - 1), (x / 2, y / 2))
    };
    if path_step(a, b) {
        Cell::Path
    } else if has(EXPANDED, a) && has(EXPANDED, b) {
        Cell::Expanded
    } else {
        Cell::Floor
    }
}

/// Resolve one display-grid position to a generation cell state.
pub fn gen_at(x: usize, y: usize) -> GenCell {
    let ch = GEN_GRID[y].as_bytes()[x];
    if ch == b'#' {
        return GenCell::Uncarved;
    }
    if x % 2 == 1 && y % 2 == 1 {
        let p = (x / 2, y / 2);
        if p == GEN_CURRENT {
            return GenCell::Current;
        }
        if has(GEN_STACK, p) {
            return GenCell::Frontier;
        }
        if has(GEN_CARVED, p) {
            return GenCell::Carved;
        }
        return GenCell::Uncarved;
    }
    GenCell::Carved
}

// ---------------------------------------------------------------- statistics

pub struct RunStats {
    pub algorithm: &'static str,
    pub steps: u32,
    pub expanded: u32,
    pub frontier: u32,
    pub path_len: u32,
}

/// The run now on screen: A*, solved.
pub const CURRENT_RUN: RunStats = RunStats {
    algorithm: "A*",
    steps: 52,
    expanded: 52,
    frontier: 5,
    path_len: 34,
};

/// The run kept on screen beside it, as sequential comparison requires.
pub const PREVIOUS_RUN: RunStats = RunStats {
    algorithm: "BFS",
    steps: 64,
    expanded: 64,
    frontier: 0,
    path_len: 34,
};

pub const GEN_STEPS: u32 = 45;
pub const GEN_CARVED_COUNT: u32 = 29;
pub const GEN_FRONTIER_COUNT: u32 = 12;
pub const GENERATOR: &str = "Recursive Backtracker";
pub const BRAID: f32 = 0.0;

/// The speed ladder of decision 11, and the rung now selected.
pub const SPEED_LADDER: &[&str] = &["0.5", "1", "2", "4", "8", "16", "64", "256", "1024"];
pub const SPEED_RUNG: usize = 4;
