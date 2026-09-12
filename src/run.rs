//! `Run`: the start, the goal, the statistics and cell-state resolution.
//!
//! A **run** is one execution of a generator or a solver over a maze, together
//! with its statistics. A maze outlives the runs made on it.
//!
//! A run owns the algorithm it is an execution of. That is what lets
//! [`Run::solver_state`] answer from `&self` and a maze alone: the priority
//! walk of section 7.3 asks the solver for the current cell, the path, the
//! frontier and the expanded flags, and a run that only held numbers could not.
//!
//! **These functions resolve to a cell state, not to a colour.** That is why
//! they live here and not in [`crate::ui`], and it is why `--ascii` costs
//! nothing: the same cell state, a different glyph table.

use crate::StepOutcome;
use crate::generator::{Generator, GeneratorEntry};
use crate::maze::{Cell, DisplayCell, Maze};
use crate::rng::Rng;
use crate::solver::{Solver, SolverEntry};

/// The state a cell resolves to while a solver runs.
///
/// **The variant order is the priority order, and earlier wins.** A cell can
/// match more than one condition at once: the start cell is expanded from the
/// first step onward, and every cell on the path was expanded to get there.
///
/// The derived `Ord` follows that order, so the **weaker** of two states, which
/// is the one later in the order, is `a.max(b)`. That is the whole of the
/// passage rule of section 7.3.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SolverCellState {
    /// The cell the solver starts from.
    Start,
    /// The cell the solver looks for.
    Goal,
    /// The single cell the step now shown is acting on.
    Current,
    /// A cell on the path the solver returned.
    Path,
    /// A cell in the solver's working set.
    Frontier,
    /// A cell the solver took out of its frontier and processed.
    Expanded,
    /// A cell of the maze that the run has not reached.
    Open,
    /// A wall, or a post where two walls meet.
    Wall,
}

/// The state a cell resolves to while a generator runs.
///
/// **The variant order is the priority order, and earlier wins**, and the
/// derived `Ord` follows it, exactly as [`SolverCellState`] does. The cell the
/// step is acting on is carved as well, and it shows as `Current`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum GenCellState {
    /// The single cell the step now shown is acting on.
    Current,
    /// A cell in the generator's working set.
    Frontier,
    /// A cell the generator has attached to the maze.
    Carved,
    /// A cell the generator has not reached, and every wall.
    Uncarved,
}

/// What the statistics band of section 8 draws for one run.
///
/// One shape for both kinds of run. `expanded_or_carved` is the count that
/// carries the comparison of ADR 0009: **expanded** for a solver run, the
/// **carved count** for a generation run, and the band labels the row for the
/// run it is drawing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RunStats {
    /// The abbreviated name, which is the one the band has room for.
    pub algorithm_short: &'static str,
    /// The number of calls to [`Run::step`] this run has taken.
    pub steps: u64,
    /// The cells a solver expanded, or the cells a generator carved.
    pub expanded_or_carved: usize,
    /// The live size of the working set.
    pub frontier: usize,
    /// The cells from start to goal inclusive. `None` for a generation run,
    /// and for a solver run that has not reached the goal.
    pub path_len: Option<usize>,
}

/// The algorithm a run is an execution of.
///
/// The run owns it. A generator takes the maze by `&mut` and a solver takes it
/// by `&`, which is why they are two traits and not one, and this enum is
/// where that difference is absorbed for the loop. See ADR 0003, two traits
/// not one.
enum Algorithm {
    /// A run that carves the maze.
    Generation(Box<dyn Generator>),
    /// A run that searches the finished maze for a path.
    Solving(Box<dyn Solver>),
}

/// One execution of a generator or a solver over a maze, with its statistics.
pub struct Run {
    /// The algorithm this run is an execution of.
    algorithm: Algorithm,
    /// The cell the solver starts from. Section 15 fixes it at a corner and
    /// holds it here, as data on the run.
    start: Cell,
    /// The cell the solver looks for, at the opposite corner.
    goal: Cell,
    /// The abbreviated name of the algorithm, for the statistics band.
    algorithm_short: &'static str,
    /// The number of calls to [`Run::step`]. A **step** is one such call, and
    /// a step that only moved a working set is a step too.
    steps: u64,
    /// One flag for each cell: the cell is on the path. The path arrives as a
    /// slice on the step that reached the goal, and this is that slice as a
    /// grid.
    ///
    /// The renderer resolves one position at a time, so the walk of section
    /// 7.3 has to answer in O(1). That is rule 5 of the solver contract, and a
    /// scan of the path slice for every position of the display grid would
    /// break it.
    path_flags: Vec<bool>,
}

impl Run {
    /// Makes a generation run over `maze`, drawing from `rng`.
    ///
    /// The maze is untouched until the first [`Run::step`], which is rule 4 of
    /// the generator contract.
    #[must_use]
    pub fn generating(maze: &Maze, entry: &GeneratorEntry, rng: &mut Rng) -> Self {
        let (start, goal) = corners(maze);
        Self {
            algorithm: Algorithm::Generation((entry.make)(maze, rng)),
            start,
            goal,
            algorithm_short: entry.short,
            steps: 0,
            path_flags: Vec::new(),
        }
    }

    /// Makes a solver run over `maze`, from one corner to the other.
    #[must_use]
    pub fn solving(maze: &Maze, entry: &SolverEntry) -> Self {
        let (start, goal) = corners(maze);
        Self {
            algorithm: Algorithm::Solving((entry.make)(maze, start, goal)),
            start,
            goal,
            algorithm_short: entry.short,
            steps: 0,
            path_flags: vec![false; usize::from(maze.width()) * usize::from(maze.height())],
        }
    }

    /// Advances the run one step.
    ///
    /// Returns [`StepOutcome::Done`] on the step that completes the run. The
    /// caller must not step again after that.
    pub fn step(&mut self, maze: &mut Maze) -> StepOutcome {
        self.steps += 1;
        let outcome = match &mut self.algorithm {
            Algorithm::Generation(generator) => generator.step(maze),
            // The maze is reborrowed as `&Maze` here, because that is what
            // [`Solver::step`] takes. One signature over both arms is what
            // lets the loop of section 6.1 advance either kind the same way,
            // and it costs nothing: a solver still cannot carve, whatever the
            // caller holds. See ADR 0003, two traits not one.
            Algorithm::Solving(solver) => solver.step(maze),
        };
        if outcome == StepOutcome::Done {
            self.fill_path_flags(maze);
        }
        outcome
    }

    /// Writes the path a solver returned into the flag grid.
    ///
    /// The step that reached the goal is the one step this runs after, and a
    /// generation run leaves the grid empty.
    fn fill_path_flags(&mut self, maze: &Maze) {
        // The grid is taken out of `self` for the walk, because the path is
        // borrowed out of `self.algorithm` and the two would otherwise be a
        // shared and a unique borrow of one value at once.
        let mut flags = std::mem::take(&mut self.path_flags);
        if let Algorithm::Solving(solver) = &self.algorithm
            && let Some(path) = solver.path()
        {
            for &c in path {
                if let Some(i) = offset(maze, c) {
                    flags[i] = true;
                }
            }
        }
        self.path_flags = flags;
    }

    /// True when the cell is on the path the solver returned.
    fn is_on_path(&self, maze: &Maze, c: Cell) -> bool {
        offset(maze, c).is_some_and(|i| self.path_flags[i])
    }

    /// The cell the run starts from.
    #[must_use]
    pub const fn start(&self) -> Cell {
        self.start
    }

    /// The cell a solver run looks for.
    #[must_use]
    pub const fn goal(&self) -> Cell {
        self.goal
    }

    /// What the statistics band of section 8 draws for this run.
    ///
    /// The **carved count** is read from the maze, because that is where a
    /// generator puts it: `mark_carved` and `carve` both attach a cell, and
    /// counting them twice in a generator would be a second source of one
    /// number.
    #[must_use]
    pub fn stats(&self, maze: &Maze) -> RunStats {
        let (expanded_or_carved, frontier, path_len) = match &self.algorithm {
            Algorithm::Generation(generator) => {
                (maze.carved_count(), generator.frontier_len(), None)
            }
            Algorithm::Solving(solver) => (
                solver.expanded_count(),
                solver.frontier_len(),
                solver.path().map(<[Cell]>::len),
            ),
        };
        RunStats {
            algorithm_short: self.algorithm_short,
            steps: self.steps,
            expanded_or_carved,
            frontier,
            path_len,
        }
    }

    /// The solver this run is an execution of, or `None` for a generation run.
    fn solver(&self) -> Option<&dyn Solver> {
        match &self.algorithm {
            Algorithm::Solving(solver) => Some(&**solver),
            Algorithm::Generation(_) => None,
        }
    }

    /// The generator this run is an execution of, or `None` for a solver run.
    fn generator(&self) -> Option<&dyn Generator> {
        match &self.algorithm {
            Algorithm::Generation(generator) => Some(&**generator),
            Algorithm::Solving(_) => None,
        }
    }

    /// The cell state at the display position `(dx, dy)`, while a solver runs.
    ///
    /// A [`DisplayCell::Cell`] walks the priority order from the top. A
    /// [`DisplayCell::Passage`] takes the weaker of its two ends, each end
    /// resolved to its non-positional state. A [`DisplayCell::Wall`], and
    /// every position outside the display grid, is [`SolverCellState::Wall`].
    ///
    /// Section 7.3 gives one function for each phase, and the phase decides
    /// which one the screen calls. **A call on a run of the other kind is a
    /// bug in the caller**, and the debug assertion below is what catches it.
    /// A release build answers `Open` for every cell of the maze instead of
    /// taking the application down, because section 10 has a panic tear the
    /// terminal down and restore it, and one wrongly drawn frame does not earn
    /// that.
    #[must_use]
    pub fn solver_state(&self, maze: &Maze, dx: u16, dy: u16) -> SolverCellState {
        debug_assert!(
            self.solver().is_some(),
            "solver_state asks a generation run for a solver cell state"
        );
        match maze.display_cell(dx, dy) {
            DisplayCell::Cell(c) => {
                if c == self.start {
                    SolverCellState::Start
                } else if c == self.goal {
                    SolverCellState::Goal
                } else if self.solver().and_then(Solver::current) == Some(c) {
                    SolverCellState::Current
                } else {
                    self.solver_body(maze, c)
                }
            }
            DisplayCell::Passage(a, b) => self.solver_body(maze, a).max(self.solver_body(maze, b)),
            DisplayCell::Wall => SolverCellState::Wall,
        }
    }

    /// The non-positional state of a cell: the priority walk with `Start`,
    /// `Goal` and `Current` skipped, so it begins at `Path`.
    ///
    /// A passage resolves both of its ends this way. The reason is contiguity
    /// without inflation. A solved path has to read as one connected line, so
    /// a passage between two path cells is `Path`. The three states this walk
    /// skips are the ones that belong to a single cell and would read as a
    /// second start, a second goal or a second current cell if a passage took
    /// them.
    fn solver_body(&self, maze: &Maze, c: Cell) -> SolverCellState {
        let Some(solver) = self.solver() else {
            return SolverCellState::Open;
        };
        if self.is_on_path(maze, c) {
            SolverCellState::Path
        } else if solver.is_frontier(c) {
            SolverCellState::Frontier
        } else if solver.is_expanded(c) {
            SolverCellState::Expanded
        } else {
            SolverCellState::Open
        }
    }

    /// The cell state at the display position `(dx, dy)`, while a generator
    /// runs.
    ///
    /// The walk is the one [`Run::solver_state`] makes, over the four states
    /// of a generation run. `Current` is the one state a passage skips, and
    /// `Carved` comes from the maze and not from the generator.
    ///
    /// A call on a solver run is a bug in the caller, for the reason
    /// [`Run::solver_state`] gives, and a release build answers from the maze
    /// alone: every carved cell is `Carved`.
    #[must_use]
    pub fn gen_state(&self, maze: &Maze, dx: u16, dy: u16) -> GenCellState {
        debug_assert!(
            self.generator().is_some(),
            "gen_state asks a solver run for a generation cell state"
        );
        match maze.display_cell(dx, dy) {
            DisplayCell::Cell(c) => {
                if self.generator().and_then(Generator::current) == Some(c) {
                    GenCellState::Current
                } else {
                    self.gen_body(maze, c)
                }
            }
            DisplayCell::Passage(a, b) => self.gen_body(maze, a).max(self.gen_body(maze, b)),
            DisplayCell::Wall => GenCellState::Uncarved,
        }
    }

    /// The non-positional generation state of a cell: the walk with `Current`
    /// skipped, so it begins at `Frontier`.
    ///
    /// What this gives a passage differs between the two generators, because
    /// they differ in what their frontier holds. The frontier of Randomized
    /// Prim holds uncarved cells, and a passage joins two carved cells, so
    /// every passage of that run is `Carved`, as section 7.3 states. The
    /// frontier of the Recursive Backtracker is its stack and a stack cell is
    /// carved, so the corridor it is walking reads as one `Frontier` line.
    /// That is the contiguity the rule is for.
    fn gen_body(&self, maze: &Maze, c: Cell) -> GenCellState {
        let in_the_working_set = self
            .generator()
            .is_some_and(|generator| generator.is_frontier(c));
        if in_the_working_set {
            GenCellState::Frontier
        } else if maze.is_carved(c) {
            GenCellState::Carved
        } else {
            GenCellState::Uncarved
        }
    }
}

/// The offset of a cell in a grid that holds one flag for each cell of `maze`,
/// or `None` when the cell is outside it.
///
/// The column has to be checked as well as the row: an offset past the end of
/// a row lands on the next one.
fn offset(maze: &Maze, c: Cell) -> Option<usize> {
    maze.contains(c)
        .then(|| usize::from(c.y) * usize::from(maze.width()) + usize::from(c.x))
}

/// The two opposite corners of a maze: the top left and the bottom right.
///
/// Section 15 fixes the start and the goal there for v1, and holds them as
/// data on the run rather than inside a solver, so that moving them later is a
/// change to this function and to the keymap alone.
fn corners(maze: &Maze) -> (Cell, Cell) {
    (
        Cell { x: 0, y: 0 },
        Cell {
            x: maze.width().saturating_sub(1),
            y: maze.height().saturating_sub(1),
        },
    )
}
