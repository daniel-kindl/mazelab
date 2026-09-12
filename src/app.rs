//! `App`: the phases, the selection, the speed and the size. The state machine.
//!
//! `App` owns the model and the selection. It holds **no terminal and no
//! clock**: the loop of section 6 measures the time and passes a [`Duration`]
//! in, `main.rs` translates a key to an [`Action`] and passes that in, and the
//! renderer reads the size from the frame. This is the one-way rule of ADR
//! 0006 applied to the state machine.
//!
//! Three seams follow from that rule, and each one is where a test can reach:
//!
//! - [`capacity`] is a pure function of two integers, so the size rules of
//!   section 13.2 are testable without a terminal.
//! - [`StepBudget`] holds the ladder arithmetic and takes a [`Duration`], so
//!   the speed rules of section 6.4 are testable without a clock.
//! - [`App::apply`] takes an [`Action`] and answers whether the model changed,
//!   so every transition of section 5.1 is one call.

use std::time::Duration;

use crate::StepOutcome;
use crate::braid::braid;
use crate::generator::GENERATORS;
use crate::maze::Maze;
use crate::rng::{self, Rng};
use crate::run::{Run, RunStats};
use crate::solver::SOLVERS;

/// What a [`Phase::Paused`] paused.
///
/// A resume has to return to the activity it stopped, and this is what carries
/// it. Section 13.4 reuses the same pair for auto-pause, which is why
/// auto-pause needs no new state.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Activity {
    /// A run that carves the maze.
    Generating,
    /// A run that searches the finished maze for a path.
    Solving,
}

impl Activity {
    /// The phase this activity is in while it animates.
    ///
    /// It is the inverse of [`Phase::activity`] over the two animating
    /// phases, and it is what a resume returns to.
    #[must_use]
    pub const fn animating(self) -> Phase {
        match self {
            Self::Generating => Phase::Generating,
            Self::Solving => Phase::Solving,
        }
    }
}

/// The state that decides what the application does with a tick.
///
/// A phase belongs to the application, not to an algorithm: an algorithm holds
/// its own working set and knows nothing about pausing. Section 5.1 gives the
/// transitions and [`App::apply`] is where they are written.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Before the first maze. Section 13.2 has the application generate one at
    /// startup, so [`App::new`] passes through this phase and does not stop in
    /// it. [`App::idle`] is what stops in it.
    Idle,
    /// A generation run is animating.
    Generating,
    /// A run is stopped, and holds what it stopped.
    Paused(Activity),
    /// A finished maze, with no solver run on it.
    Ready,
    /// A solver run is animating.
    Solving,
    /// A solver run reached the goal.
    Solved,
}

impl Phase {
    /// True while a run advances with the tick.
    ///
    /// The loop of section 6.1 reads this twice: it sizes the `poll` timeout,
    /// and it decides whether to spend a step budget. A paused application
    /// draws nothing and blocks in `poll` for a second at a time.
    #[must_use]
    pub const fn is_animating(self) -> bool {
        matches!(self, Self::Generating | Self::Solving)
    }

    /// What this phase is a run of, or `None` where there is no run to
    /// advance.
    ///
    /// A paused phase answers what it paused, so the phases that hold a run
    /// answer the same whether the run is animating or stopped. Pause, single
    /// step, auto-pause and the end of a run all ask this one question, and
    /// this is where the answer lives.
    #[must_use]
    pub const fn activity(self) -> Option<Activity> {
        match self {
            Self::Generating => Some(Activity::Generating),
            Self::Solving => Some(Activity::Solving),
            Self::Paused(activity) => Some(activity),
            Self::Idle | Self::Ready | Self::Solved => None,
        }
    }
}

/// Which of the two thresholds of section 13 fired.
///
/// **There are two, not one.** They have different causes, different remedies
/// and different wordings, and they must not be collapsed. The caller decides
/// which one fired, because the decision needs the terminal size; `App` learns
/// the answer and uses it for one thing, which is the auto-pause of section
/// 13.4.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TooSmall {
    /// The maze is larger than the terminal can show. The maze pane is
    /// replaced; the status row, the statistics band, the legend and the help
    /// row keep rendering. The remedy is a resize or `f`.
    MazeFit,
    /// The terminal is below the layout floor of 79 x 19. The whole screen is
    /// replaced, and the only remedy is a resize.
    LayoutFloor,
}

/// What a key does, with the key left behind.
///
/// `main.rs` translates a `KeyEvent` to one of these and `app` never sees a
/// key code. That is the rule of section 1.2, and it is what lets every
/// transition of section 5.1 be tested with no terminal.
///
/// Section 7.5 is the keymap, and every row of it is here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    /// `g`: build a new maze and animate the generation.
    Generate,
    /// `G`: build a new maze and finish it at once. Section 6.5.
    GenerateInstantly,
    /// `s`: run the selected solver.
    Solve,
    /// `Space`: pause a run, or resume the run that is paused.
    PauseResume,
    /// `.`: advance exactly one step, whatever the speed. It implies pause.
    SingleStep,
    /// `+`: one rung up the ladder.
    Faster,
    /// `-`: one rung down the ladder.
    Slower,
    /// `1` and `2`: select the generator for the next generate. It does not
    /// generate.
    ChooseGenerator(usize),
    /// `3`, `4` and `5`: select the solver.
    ChooseSolver(usize),
    /// `Tab`: select the next solver, and wrap.
    NextSolver,
    /// `<-`: one cell narrower.
    Narrower,
    /// `->`: one cell wider.
    Wider,
    /// `Up`: one cell taller.
    Taller,
    /// `Down`: one cell shorter.
    Shorter,
    /// `f`: fit the maze to capacity and regenerate. Section 13.3.
    Refit,
    /// `n`: a fresh random seed, then a new maze.
    NewSeed,
    /// `b`: the next braid factor, then a new maze.
    CycleBraid,
    /// `?`: toggle the help overlay.
    ToggleHelp,
    /// `Esc`: close the help overlay, or quit when the overlay is closed.
    ///
    /// The key is `Esc` and the action is what section 7.5 has it do, because
    /// which of the two it does depends on the model and the model lives here.
    Dismiss,
    /// `q`: quit. The overlay does not hold it back.
    Quit,
}

/// The smallest maze MazeLab shows. Section 13.1: at three rows a maze is a
/// corridor and backtracking has nowhere to happen.
pub const MIN_SIZE: u16 = 4;

/// The hard cap on each axis, reachable only from the command line, where
/// section 9 rejects a larger value at parse time instead of clamping it.
pub const MAX_SIZE: u16 = 512;

/// The columns of the layout floor, set by the short form of the legend, which
/// is the widest thing MazeLab draws.
pub const FLOOR_COLS: u16 = 79;

/// The rows of the layout floor: [`CHROME_ROWS`] plus the 9 rows a 4-cell-tall
/// maze needs.
pub const FLOOR_ROWS: u16 = 19;

/// The rows the chrome costs: the status row, the bordered statistics band,
/// the legend row and the help row. Section 7.1. The chrome costs no columns.
pub const CHROME_ROWS: u16 = 10;

/// The braid factors that `b` cycles. Section 7.5.
pub const BRAID_FACTORS: [f64; 3] = [0.00, 0.25, 0.50];

/// The largest maze a terminal of `cols` x `rows` can show, or `None` below
/// the layout floor, where capacity is undefined.
///
/// A maze needs `2W + 1` columns of display grid at two screen columns each,
/// plus one column of slack, and `2H + 1` rows plus the chrome. Section 7.1
/// states the result:
///
/// ```text
/// W = (cols - 2) / 4          H = (rows - 11) / 2
/// ```
///
/// **Capacity is a property of the terminal, not of the maze.** A maze can be
/// larger than capacity, and then it is not shown: that is the maze-fit
/// threshold of section 13, and it is the caller's to detect. Capacity is here
/// and not in [`crate::ui`] because the model needs it twice, for the startup
/// chain and for the clamp of section 13.2, and because it is a pure function
/// of two integers with no terminal in it.
///
/// The result is held at [`MAX_SIZE`], which a maze may not pass. Section 13.1
/// states that capacity can never reach 512, so the bound is unreachable on a
/// terminal and is here to keep an absurd one inside what [`Maze::new`]
/// accepts.
#[must_use]
pub fn capacity(cols: u16, rows: u16) -> Option<(u16, u16)> {
    if cols < FLOOR_COLS || rows < FLOOR_ROWS {
        return None;
    }
    let width = (cols - 2) / 4;
    let height = (rows - CHROME_ROWS - 1) / 2;
    Some((width.min(MAX_SIZE), height.min(MAX_SIZE)))
}

/// The speed ladder: one fixed, non-linear list that spans generation and
/// solving.
///
/// A rung is stated in steps per frame at the nominal 60 Hz redraw rate, so
/// the 0.5 rung is 30 steps per second.
pub const LADDER: [f64; 9] = [0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 64.0, 256.0, 1024.0];

/// The rung the application starts on: 8 steps per frame.
pub const DEFAULT_RUNG: usize = 4;

/// The nominal redraw rate that a rung is stated against.
const NOMINAL_HZ: f64 = 60.0;

/// The number of steps a tick may run, and the rung it comes from.
///
/// **There is no [`std::time::Instant`] inside this type.** The loop measures
/// the time and passes a [`Duration`] in. The ladder and residue arithmetic is
/// the part of the loop most likely to be wrong, and section 11.4 forbids
/// driving the real event loop, so this is where the seam goes. The deadline
/// and the discard of section 6.3 stay in the loop, because they need a clock
/// read in the middle of a batch.
///
/// The budget follows the clock and not a count of ticks, and a budget below
/// one step accumulates in the **residue** until it reaches one.
#[derive(Clone, Copy, Debug)]
pub struct StepBudget {
    /// The index into [`LADDER`].
    rung: usize,
    /// The part of a step the ticks so far bought and did not spend. It is
    /// always below one step.
    residue: f64,
}

impl StepBudget {
    /// A budget on [`DEFAULT_RUNG`], with nothing owed.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            rung: DEFAULT_RUNG,
            residue: 0.0,
        }
    }

    /// The steps `elapsed` buys, with the fraction left over kept.
    ///
    /// The caller clamps the elapsed time to 100 ms before the call, which is
    /// the rule of section 6.3 and covers a suspend, a sleep and a debugger
    /// breakpoint.
    pub fn advance(&mut self, elapsed: Duration) -> u32 {
        self.residue += elapsed.as_secs_f64() * LADDER[self.rung] * NOMINAL_HZ;
        let whole = self.residue.floor();
        self.residue -= whole;
        // A floored, non-negative count. The cast saturates at `u32::MAX`, and
        // the 100 ms clamp of section 6.3 holds one tick near 6 000 steps at
        // the top rung. `Cargo.toml` holds the two casting lints this needs.
        whole as u32
    }

    /// Drops the unspent budget, which is what the loop does at the deadline
    /// of section 6.3.
    ///
    /// Carrying the budget instead guarantees that the next tick overruns too,
    /// and the debt compounds. Dropping it is the honest failure: the
    /// animation runs slower than the requested rung because the machine
    /// cannot go faster, and it recovers as soon as the load drops.
    pub const fn discard(&mut self) {
        self.residue = 0.0;
    }

    /// Starts the budget clean, which is what a resume and a single step do.
    ///
    /// Section 6.4 gives the reason for both: a resume that kept ten seconds
    /// of budget would make the algorithm jump, and single step is exact and
    /// should not leave a fraction that makes the next automatic step arrive
    /// early.
    pub const fn reset(&mut self) {
        self.residue = 0.0;
    }

    /// One rung up. It saturates at the top of the ladder, and answers whether
    /// the rung moved.
    ///
    /// **The residue is kept.** It is under one step by definition, so
    /// dropping it is unobservable, and a rule with no effect is a rule to get
    /// wrong later.
    ///
    /// The answer is what keeps the redraw rule of section 6.1 honest: at the
    /// end of the ladder the status row does not change, so the key costs no
    /// frame.
    pub const fn faster(&mut self) -> bool {
        if self.rung + 1 >= LADDER.len() {
            return false;
        }
        self.rung += 1;
        true
    }

    /// One rung down. It saturates at the bottom of the ladder, keeps the
    /// residue, and answers whether the rung moved, for the reasons
    /// [`StepBudget::faster`] gives.
    pub const fn slower(&mut self) -> bool {
        if self.rung == 0 {
            return false;
        }
        self.rung -= 1;
        true
    }

    /// The rung, in steps per frame. The status row prints it as `0.5x`
    /// through `1024x`.
    #[must_use]
    pub const fn rung(&self) -> f64 {
        LADDER[self.rung]
    }
}

impl Default for StepBudget {
    fn default() -> Self {
        Self::new()
    }
}

/// What the command line settled before the application started.
///
/// Section 9 parses it and section 13.2 gives the size chain: the flag if
/// given, else capacity, else the 4 x 4 minimum where the terminal is below
/// the layout floor and capacity is undefined. The chain is applied for each
/// axis on its own, so a pair with one flag given uses capacity for the other.
#[derive(Clone, Copy, Debug)]
pub struct Startup {
    /// The seed every random draw of a generate comes from.
    pub seed: u64,
    /// `--width`, or `None` for the fallback chain.
    pub width: Option<u16>,
    /// `--height`, or `None` for the fallback chain.
    pub height: Option<u16>,
    /// `--ascii`: the glyph set that holds no character above `U+007F`.
    pub ascii: bool,
    /// The index into `GENERATORS` that `--generator` named.
    pub generator_ix: usize,
    /// The index into `SOLVERS` that `--solver` named.
    pub solver_ix: usize,
}

/// The model, the selection and the phase: the state machine of section 5.
///
/// The two index fields hold the **selection**, which is what the next `g` or
/// `s` runs. [`App::run`] holds what is running, together with the algorithm
/// box, because the cell-state functions of section 7.3 take the maze and
/// nothing else.
pub struct App {
    /// What the application does with a tick.
    pub phase: Phase,
    /// The number that fixes every random draw a generate makes.
    pub seed: u64,
    /// The maze on screen. A maze outlives the runs made on it.
    pub maze: Maze,
    /// The fraction of dead ends that braiding removes.
    pub braid_factor: f64,
    /// The index into `GENERATORS` that the next `g` runs.
    pub generator_ix: usize,
    /// The index into `SOLVERS` that the next `s` runs.
    pub solver_ix: usize,
    /// The run on screen, and the algorithm it is an execution of.
    pub run: Option<Run>,
    /// The previous-run column of section 8: at most one **finished** solver
    /// run, filled when the phase leaves [`Phase::Solved`].
    pub previous: Option<RunStats>,
    /// The steps a tick may run.
    pub budget: StepBudget,
    /// The ASCII glyph set is selected.
    pub ascii: bool,
    /// The help overlay is open over the maze pane.
    pub help_open: bool,
    /// Which threshold of section 13 the caller last reported.
    pub too_small: Option<TooSmall>,
    /// The loop of section 6.1 breaks on this.
    ///
    /// Section 5.3 does not list the field and section 6.1 reads it. The loop
    /// is what owns the terminal, so quitting has to reach it as data.
    pub quit: bool,
    /// The largest maze the terminal can show, as the caller last computed it,
    /// or `None` below the layout floor.
    ///
    /// **`App` never learns the terminal size.** It learns capacity, which is
    /// the one thing the size rules of section 13.2 need, and the caller
    /// computes it with [`capacity`].
    capacity: Option<(u16, u16)>,
    /// The RNG of the maze on screen, at the point generation reached.
    ///
    /// Section 5.3 does not list the field, and the braiding pass of section
    /// 4.3 needs it: the pass runs when the generation run reports `Done`,
    /// which is one or more ticks after the generate started, and it must draw
    /// from the stream the generate started on. A generator runs on a child of
    /// this stream, taken with [`rng::split`], so the two never overlap and
    /// the seed still reproduces the braided maze. ADR 0005.
    rng: Rng,
}

impl App {
    /// The application before its first maze, at [`Phase::Idle`].
    ///
    /// Section 13.2 has a maze generated at startup, whatever the terminal
    /// size, so [`App::new`] is what the application uses. This constructor is
    /// the state the phase diagram of section 5.1 names `Idle`, and it is what
    /// the transitions out of `Idle` are tested from.
    #[must_use]
    pub fn idle(startup: Startup, capacity: Option<(u16, u16)>) -> Self {
        let (width, height) = startup_size(startup, capacity);
        Self {
            phase: Phase::Idle,
            seed: startup.seed,
            maze: Maze::new(width, height),
            braid_factor: BRAID_FACTORS[0],
            generator_ix: startup.generator_ix.min(GENERATORS.len() - 1),
            solver_ix: startup.solver_ix.min(SOLVERS.len() - 1),
            run: None,
            previous: None,
            budget: StepBudget::new(),
            ascii: startup.ascii,
            help_open: false,
            too_small: None,
            quit: false,
            capacity,
            rng: rng::from_seed(startup.seed),
        }
    }

    /// The application at startup, with a maze already generated.
    ///
    /// **A maze is always generated at startup**, whatever the terminal size.
    /// A later resize reveals a maze that already exists; it never triggers
    /// one. Section 13.2.
    #[must_use]
    pub fn new(startup: Startup, capacity: Option<(u16, u16)>) -> Self {
        let mut app = Self::idle(startup, capacity);
        let (width, height) = app.size();
        app.generate_instantly(width, height);
        app
    }

    /// Applies one action and answers whether the model changed.
    ///
    /// The answer is the dirty flag of section 6.1: the loop redraws only when
    /// the model changed, so an action that is inert costs no frame. Section
    /// 13.2 has a size key at the clamp be silently inert, and this is where
    /// that shows.
    pub fn apply(&mut self, action: Action) -> bool {
        let (width, height) = self.size();
        match action {
            Action::Generate => {
                self.generate(width, height);
                true
            }
            Action::GenerateInstantly => {
                self.generate_instantly(width, height);
                true
            }
            Action::NewSeed => {
                self.seed = rng::fresh_seed();
                self.generate_instantly(width, height);
                true
            }
            Action::CycleBraid => {
                self.braid_factor = next_braid_factor(self.braid_factor);
                self.generate_instantly(width, height);
                true
            }
            Action::Narrower => self.resize(shrunk(width), height),
            Action::Wider => {
                let limit = limit(width, self.capacity.map(|(width, _)| width));
                self.resize(grown(width, limit), height)
            }
            Action::Shorter => self.resize(width, shrunk(height)),
            Action::Taller => {
                let limit = limit(height, self.capacity.map(|(_, height)| height));
                self.resize(width, grown(height, limit))
            }
            Action::Refit => self.refit(),
            Action::Solve => self.solve(),
            Action::PauseResume => self.pause_resume(),
            Action::SingleStep => self.single_step(),
            Action::Faster => self.budget.faster(),
            Action::Slower => self.budget.slower(),
            Action::ChooseGenerator(ix) => self.choose_generator(ix),
            Action::ChooseSolver(ix) => self.choose_solver(ix),
            Action::NextSolver => self.choose_solver((self.solver_ix + 1) % SOLVERS.len()),
            Action::ToggleHelp => {
                self.help_open = !self.help_open;
                true
            }
            Action::Dismiss => {
                if self.help_open {
                    self.help_open = false;
                } else {
                    self.quit = true;
                }
                true
            }
            Action::Quit => {
                self.quit = true;
                true
            }
        }
    }

    /// Advances the run on screen one step.
    ///
    /// The loop of section 6.1 calls this for each step of the budget, and
    /// calls [`App::advance_phase`] on [`StepOutcome::Done`]. A phase with no
    /// run answers `Done`, so a caller that reached here in a still phase
    /// stops at once instead of stepping a run that is not there.
    pub fn step(&mut self) -> StepOutcome {
        let Some(run) = self.run.as_mut() else {
            return StepOutcome::Done;
        };
        run.step(&mut self.maze)
    }

    /// Takes the phase to where a finished run leaves it.
    ///
    /// A generation run is followed by the braiding pass, then
    /// [`Phase::Ready`]. The pass is instant, so `Generating` never leads
    /// anywhere else. A solver run reaches [`Phase::Solved`].
    ///
    /// A paused run that was single-stepped to its end arrives here too, which
    /// is why the paused phases are matched beside the animating ones.
    pub fn advance_phase(&mut self) {
        match self.phase.activity() {
            Some(Activity::Generating) => {
                braid(&mut self.maze, self.braid_factor, &mut self.rng);
                self.phase = Phase::Ready;
            }
            Some(Activity::Solving) => self.phase = Phase::Solved,
            None => {}
        }
    }

    /// Learns the largest maze the terminal can show.
    ///
    /// The caller computes it with [`capacity`] and passes the result in. It
    /// changes no maze on its own: section 13.2 has a resize reveal a maze
    /// that already exists, and never generate one.
    pub const fn set_capacity(&mut self, capacity: Option<(u16, u16)>) {
        self.capacity = capacity;
    }

    /// Learns which threshold of section 13 fired, and auto-pauses a run that
    /// entered one.
    ///
    /// **Growing the terminal back does not auto-resume.** The maze reappears,
    /// still paused, and `Space` continues. MazeLab exists to make an
    /// algorithm visible one step at a time, so stepping where nobody can see
    /// is the one outcome with no value; and auto-resume would make a dragged
    /// window edge fire a stutter of pause and resume. Section 13.4.
    pub fn set_too_small(&mut self, too_small: Option<TooSmall>) {
        self.too_small = too_small;
        if too_small.is_some()
            && let Some(activity) = self.phase.activity()
        {
            // A phase that is already paused pauses to itself, so entering the
            // second threshold from the first changes nothing.
            self.phase = Phase::Paused(activity);
        }
    }

    /// The size of the maze on screen.
    const fn size(&self) -> (u16, u16) {
        (self.maze.width(), self.maze.height())
    }

    /// Builds a new maze and the generation run over it.
    ///
    /// **A new maze resets everything**: it clears the previous-run column and
    /// discards any solver run in progress. The comparison of ADR 0009 is
    /// between solvers on one maze, so a new maze ends it. Section 5.2.
    fn build(&mut self, width: u16, height: u16) -> Run {
        self.maze = Maze::new(width, height);
        self.rng = rng::from_seed(self.seed);
        self.previous = None;
        self.budget.reset();
        Run::generating(&self.maze, &GENERATORS[self.generator_ix], &mut self.rng)
    }

    /// Starts an animated generation. `g` is the one key that reaches it.
    fn generate(&mut self, width: u16, height: u16) {
        let run = self.build(width, height);
        self.run = Some(run);
        self.phase = Phase::Generating;
    }

    /// Runs a generation to completion outside the loop, then braids, then
    /// [`Phase::Ready`]. Section 6.5.
    ///
    /// The [`Phase::Generating`] phase is never entered. Running it inside the
    /// loop would put an infinity into a ladder that is defined as a finite
    /// list. The finished run stays on screen, so the "this run" column of
    /// section 8.2 holds the generation numbers while `Ready`.
    fn generate_instantly(&mut self, width: u16, height: u16) {
        let mut run = self.build(width, height);
        while run.step(&mut self.maze) == StepOutcome::Stepped {}
        braid(&mut self.maze, self.braid_factor, &mut self.rng);
        self.run = Some(run);
        self.phase = Phase::Ready;
    }

    /// Regenerates at a new size, or answers `false` when the size is the one
    /// on screen.
    ///
    /// **A size key is a generate.** It regenerates instantly at the current
    /// seed, and it discards the previous run like any new maze. Animated
    /// regeneration would start a run per keypress. **At the clamp a size key
    /// is silently inert**, and the `maze 19x4` field of the statistics band
    /// is the feedback. Section 13.2.
    fn resize(&mut self, width: u16, height: u16) -> bool {
        if (width, height) == self.size() {
            return false;
        }
        self.generate_instantly(width, height);
        true
    }

    /// Fits the maze to capacity and regenerates instantly.
    ///
    /// **Below the layout floor `f` is inert**, and the panel drops its
    /// `press f` line there. Clamping to 4 x 4 and regenerating would "work"
    /// and change nothing visible, because the chrome still cannot render.
    /// Section 13.3.
    fn refit(&mut self) -> bool {
        let Some((width, height)) = self.capacity else {
            return false;
        };
        self.generate_instantly(width, height);
        true
    }

    /// Starts a solver run on the maze on screen.
    ///
    /// Section 5.1 gives `s` one arm, `Ready --s--> Solving`. From
    /// [`Phase::Solved`] the way back to `Ready` is a solver key, which is
    /// also what promotes the finished run to the previous-run column.
    fn solve(&mut self) -> bool {
        if self.phase != Phase::Ready {
            return false;
        }
        self.run = Some(Run::solving(&self.maze, &SOLVERS[self.solver_ix]));
        self.budget.reset();
        self.phase = Phase::Solving;
        true
    }

    /// Selects the generator for the next generate.
    ///
    /// **Choosing a generator does not generate.** The maze on screen is not
    /// touched. Section 5.2.
    fn choose_generator(&mut self, ix: usize) -> bool {
        if ix >= GENERATORS.len() || ix == self.generator_ix {
            return false;
        }
        self.generator_ix = ix;
        true
    }

    /// Selects the solver, and settles what happens to the run on screen.
    ///
    /// **From [`Phase::Solved`] it promotes** the finished run to the
    /// previous-run column and goes to `Ready`. Choosing the solver that is
    /// already selected does the same, so `s` after it re-runs the same solver
    /// against the previous run. Section 8.2 fills the column when the phase
    /// **leaves** `Solved`, because promoting on entry would empty the column
    /// labelled "this run" exactly when the user wants to read it.
    ///
    /// **From [`Phase::Solving`] or `Paused(Solving)` it abandons the run**
    /// and goes to `Ready`. The abandoned run does **not** promote: it is
    /// incomplete, and a half-finished expanded count beside a finished one
    /// would be read as a comparison. Section 5.2.
    ///
    /// Both arms leave [`App::run`] empty, so the "this run" column is empty
    /// until `s` starts the next solver. Section 8.2 keeps that column full in
    /// the `Ready` that follows a generate, where the generation run is still
    /// on screen; the `Ready` a solver key reaches has no run to show, because
    /// the run it had is either promoted or abandoned.
    fn choose_solver(&mut self, ix: usize) -> bool {
        if ix >= SOLVERS.len() {
            return false;
        }
        let left_a_run = match self.phase {
            Phase::Solved => {
                self.previous = self.run.as_ref().map(|run| run.stats(&self.maze));
                self.run = None;
                self.phase = Phase::Ready;
                true
            }
            Phase::Solving | Phase::Paused(Activity::Solving) => {
                self.run = None;
                self.phase = Phase::Ready;
                true
            }
            Phase::Idle
            | Phase::Generating
            | Phase::Paused(Activity::Generating)
            | Phase::Ready => false,
        };
        let selected = ix != self.solver_ix;
        self.solver_ix = ix;
        left_a_run || selected
    }

    /// Pauses the run that is animating, or resumes the run that is paused.
    ///
    /// A resume starts the budget clean, which is the residue rule of section
    /// 6.4: keeping it means resuming after ten seconds buys ten seconds of
    /// budget and the algorithm jumps.
    fn pause_resume(&mut self) -> bool {
        self.phase = match self.phase {
            Phase::Paused(activity) => {
                self.budget.reset();
                activity.animating()
            }
            Phase::Generating => Phase::Paused(Activity::Generating),
            Phase::Solving => Phase::Paused(Activity::Solving),
            Phase::Idle | Phase::Ready | Phase::Solved => return false,
        };
        true
    }

    /// Advances exactly one step, whatever the speed.
    ///
    /// **Single step from a running phase implies pause.** Someone reaching
    /// for single step wants to stop and look, and a key that silently does
    /// nothing is reported as a bug. It zeroes the residue, because single
    /// step is exact and should not leave a fraction that makes the next
    /// automatic step arrive early. Sections 5.2 and 6.4.
    fn single_step(&mut self) -> bool {
        let Some(activity) = self.phase.activity() else {
            return false;
        };
        self.phase = Phase::Paused(activity);
        self.budget.reset();
        if self.step() == StepOutcome::Done {
            self.advance_phase();
        }
        true
    }
}

/// The startup size: the flag if given, else capacity, else the 4 x 4 minimum.
///
/// The stated size is **not** clamped to capacity. Growing into a panel by
/// pressing `->` is a dead end with no feedback, and an explicit `--width 60`
/// on a 40-column terminal is a stated intention that the application does not
/// overrule. Section 13.2.
///
/// It is held to `MIN_SIZE..=MAX_SIZE`, which is the bound the model itself
/// carries. Section 9 rejects a size outside it at parse time, so a valid
/// command line never reaches the bound here.
fn startup_size(startup: Startup, capacity: Option<(u16, u16)>) -> (u16, u16) {
    let (width, height) = capacity.unwrap_or((MIN_SIZE, MIN_SIZE));
    (
        startup.width.unwrap_or(width).clamp(MIN_SIZE, MAX_SIZE),
        startup.height.unwrap_or(height).clamp(MIN_SIZE, MAX_SIZE),
    )
}

/// The largest size a grow key may reach on one axis.
///
/// It is capacity, and the size on screen where capacity is undefined or
/// already exceeded. **A grow key must never shrink an oversized maze**: an
/// explicit `--width 60` on a 40-column terminal is a stated intention, and
/// `f` is the key that gives it up. Section 13.2.
fn limit(current: u16, capacity: Option<u16>) -> u16 {
    capacity.unwrap_or(current).min(MAX_SIZE).max(current)
}

/// One cell larger, held at `limit`.
const fn grown(current: u16, limit: u16) -> u16 {
    let target = current.saturating_add(1);
    if target > limit { current } else { target }
}

/// One cell smaller, held at [`MIN_SIZE`].
const fn shrunk(current: u16) -> u16 {
    if current > MIN_SIZE {
        current - 1
    } else {
        current
    }
}

/// The next braid factor on the cycle `b` walks, and back to the first after
/// the last.
///
/// The factor is compared with a tolerance and not for equality, because it is
/// a `f64` that other code may set.
fn next_braid_factor(current: f64) -> f64 {
    BRAID_FACTORS
        .into_iter()
        .find(|&factor| factor > current + 1e-9)
        .unwrap_or(BRAID_FACTORS[0])
}
