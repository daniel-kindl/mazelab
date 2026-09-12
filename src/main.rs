//! Terminal setup, terminal teardown, the terminal guard and the call into the
//! loop. Nothing else.
//!
//! The tick loop of section 6 lives here because the loop is what owns the
//! terminal: it polls the input queue, it reads the clock, and it draws. The
//! model it advances lives in [`mazelab::app`], and no application state is
//! held here. Key codes are translated to an [`Action`] at [`action`], so
//! `app` never sees a `KeyEvent`. That is the rule of section 1.2.

use std::io;
use std::time::{Duration, Instant};

use clap::Parser;
use mazelab::StepOutcome;
use mazelab::app::{Action, App, TooSmall, capacity};
use mazelab::cli::Args;
use mazelab::maze::Maze;
use mazelab::ui;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent};

/// One frame at the nominal 60 Hz redraw rate. A rung of the speed ladder is
/// stated in steps per frame against this rate.
const FRAME: Duration = Duration::from_nanos(16_666_667);

/// The largest elapsed time a tick may convert to a budget, about six frames.
///
/// **Measured elapsed time is clamped before it is converted.** The clamp
/// covers a suspend, a sleep and a debugger breakpoint, each of which would
/// otherwise buy a budget of minutes. Section 6.3.
const MAX_ELAPSED: Duration = Duration::from_millis(100);

/// How often a batch of steps reads the clock to test the deadline.
///
/// Not every step: at tens of thousands of steps per second an
/// [`Instant::now`] for each step is itself a cost. Section 6.3.
const DEADLINE_CHECK_EVERY: u32 = 64;

/// How long a still phase blocks in `poll`.
///
/// A paused application draws nothing and blocks here a second at a time. Any
/// event wakes the poll at once, so the length changes nothing the user sees;
/// it only keeps the block bounded. Section 6.2.
const IDLE_POLL: Duration = Duration::from_secs(1);

fn main() -> io::Result<()> {
    let args = Args::parse();
    // The first queued crossterm command is what enables
    // ENABLE_VIRTUAL_TERMINAL_PROCESSING on Windows. It is an undocumented
    // side effect. Read "Before you bump crossterm" in docs/adr/0007 before
    // changing the crossterm version.
    let terminal = ratatui::init();
    let _guard = TerminalGuard;
    run(terminal, args)
}

/// Restores the terminal when it is dropped.
///
/// It earns its place beside the panic hook that `ratatui::init` installs,
/// because it covers what that hook does not: an early `return` or a `?` out
/// of [`run`]. A double restore is harmless, because `ratatui::restore`
/// ignores its own errors.
///
/// MazeLab installs **no panic hook of its own**. The hook `ratatui::init`
/// installs already does the whole job, and a chained hook that adds nothing
/// is a hook to get wrong. If one is ever added it must be installed
/// **before** `ratatui::init`. Section 10.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

/// The tick loop of section 6.1.
///
/// A **tick** is one pass of this loop. It applies the input that is waiting,
/// advances the run by the step budget, and redraws if anything changed. A
/// tick is not a fixed length of time.
///
/// # Errors
///
/// Returns the error that polling, reading or drawing the terminal gave. The
/// guard in [`main`] restores the terminal on the way out.
fn run(mut terminal: DefaultTerminal, args: Args) -> io::Result<()> {
    let capacity = terminal_capacity(&terminal)?;
    let mut app = App::new(args.into_startup(), capacity);
    app.set_too_small(threshold(capacity, &app.maze));

    // The first frame is drawn before the loop. A tick draws at its end, so a
    // still phase would otherwise show nothing until the first poll timed out.
    terminal.draw(|frame| ui::render(frame, &app))?;

    let mut dirty = false;
    let mut last_step = Instant::now();
    let mut next_render_deadline = Instant::now() + FRAME;

    loop {
        // The timeout is exactly the time to the next render deadline, so the
        // tick never starves input.
        let timeout = if app.phase.is_animating() {
            next_render_deadline.saturating_duration_since(Instant::now())
        } else {
            IDLE_POLL
        };

        if event::poll(timeout)? {
            let was_animating = app.phase.is_animating();

            // The terminal is already resized when the event that reports the
            // resize is queued, so the size read here is the size every action
            // in this batch must clamp against.
            let capacity = terminal_capacity(&terminal)?;
            app.set_capacity(capacity);

            let mut changed = apply(&mut app, event::read()?);
            // Drain the queue. Without the drain a held key produces one
            // iteration of this loop for each repeat, and the animation speeds
            // up while the key is down. The drain is bounded by what is
            // already queued, so input never starves the tick. Section 6.2.
            while event::poll(Duration::ZERO)? {
                changed |= apply(&mut app, event::read()?);
            }

            // The threshold follows the terminal and the maze, and a size key
            // changed the maze inside the batch.
            let before = app.too_small;
            app.set_too_small(threshold(capacity, &app.maze));
            changed |= app.too_small != before;

            // Section 6.4: a resume resets the timestamp, because keeping it
            // means resuming after ten seconds buys ten seconds of budget and
            // the algorithm jumps. Every way into an animating phase is an
            // action, so this is where the reset belongs.
            if app.phase.is_animating() && !was_animating {
                last_step = Instant::now();
            }

            dirty |= changed;
        }

        if app.phase.is_animating() {
            let elapsed = last_step.elapsed().min(MAX_ELAPSED);
            last_step = Instant::now();
            let want = app.budget.advance(elapsed);
            // Half a frame, which leaves room for the draw. The unspent budget
            // is discarded and never carried: carrying it guarantees that the
            // next tick overruns too, and the debt compounds. Section 6.3.
            let deadline = Instant::now() + FRAME / 2;
            for i in 0..want {
                if i % DEADLINE_CHECK_EVERY == 0 && Instant::now() > deadline {
                    app.budget.discard();
                    break;
                }
                // The flag is set before the step, so the step that reports
                // `Done` marks the model dirty too. It carried the last change
                // of the run, and `advance_phase` changes the phase after it,
                // so a tick that ends a run must still draw.
                dirty = true;
                if app.step() == StepOutcome::Done {
                    app.advance_phase();
                    break;
                }
            }
        }

        // Redraw only when the model changed. Section 6.2.
        if dirty {
            terminal.draw(|frame| ui::render(frame, &app))?;
            dirty = false;
        }

        // The deadline paces the tick, and a tick that drew nothing is paced
        // by it too. Section 6.4 has the ticks below one step of budget change
        // nothing and never draw, so advancing the deadline only on a draw
        // would leave it in the past: `poll` would return at once and the loop
        // would spin until the residue reached a step. A rung of 0.5 spins for
        // most of two frames that way.
        let now = Instant::now();
        if next_render_deadline <= now {
            next_render_deadline = now + FRAME;
        }

        if app.quit {
            break;
        }
    }
    Ok(())
}

/// Applies one event and answers whether the model changed.
///
/// The answer is the dirty flag of section 6.1. An event that changes nothing
/// costs no frame.
fn apply(app: &mut App, event: Event) -> bool {
    match event {
        // The arm **reads nothing from the event**. Layout comes from
        // `Frame::area()` inside the draw callback, and `Terminal::draw`
        // autoresizes the fullscreen viewport on its own. The arm exists only
        // because a paused application blocked in `poll` would otherwise never
        // repaint after a resize. Section 6.2.
        Event::Resize(..) => true,
        // `as_key_press_event` is the filter of section 6.2. On Windows
        // crossterm delivers a press and a release for one keystroke, and most
        // keys of section 7.5 toggle or step, so without the filter every key
        // acts twice.
        other => other
            .as_key_press_event()
            .and_then(action)
            .is_some_and(|action| app.apply(action)),
    }
}

/// The action a key press means, or `None` for a key the keymap does not name.
///
/// Section 7.5 is the keymap and every row of it is here. `+` is accepted with
/// and without shift, so `=` is an alias; `-` needs no alias.
fn action(key: KeyEvent) -> Option<Action> {
    let action = match key.code {
        KeyCode::Char('g') => Action::Generate,
        KeyCode::Char('G') => Action::GenerateInstantly,
        KeyCode::Char('s') => Action::Solve,
        KeyCode::Char(' ') => Action::PauseResume,
        KeyCode::Char('.') => Action::SingleStep,
        KeyCode::Char('+' | '=') => Action::Faster,
        KeyCode::Char('-') => Action::Slower,
        KeyCode::Char('1') => Action::ChooseGenerator(0),
        KeyCode::Char('2') => Action::ChooseGenerator(1),
        KeyCode::Char('3') => Action::ChooseSolver(0),
        KeyCode::Char('4') => Action::ChooseSolver(1),
        KeyCode::Char('5') => Action::ChooseSolver(2),
        KeyCode::Tab => Action::NextSolver,
        KeyCode::Left => Action::Narrower,
        KeyCode::Right => Action::Wider,
        KeyCode::Up => Action::Taller,
        KeyCode::Down => Action::Shorter,
        KeyCode::Char('f') => Action::Refit,
        KeyCode::Char('n') => Action::NewSeed,
        KeyCode::Char('b') => Action::CycleBraid,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Esc => Action::Dismiss,
        _ => return None,
    };
    Some(action)
}

/// The largest maze this terminal can show, or `None` below the layout floor.
fn terminal_capacity(terminal: &DefaultTerminal) -> io::Result<Option<(u16, u16)>> {
    let size = terminal.size()?;
    Ok(capacity(size.width, size.height))
}

/// Which threshold of section 13 fired, or `None` when the maze fits.
///
/// **There are two thresholds, not one.** The loop decides between them and
/// `App` learns the answer, because the loop is what reads the terminal.
///
/// The layout floor is the absolute one, so it is tested first: below the
/// floor capacity is undefined, and the maze-fit question cannot be asked of a
/// capacity that does not exist.
///
/// The maze-fit test of section 13 is `cols < 4W + 2` or `rows < 2H + 11`.
/// Capacity is `(cols - 2) / 4` by `(rows - 11) / 2`, so the maze against
/// capacity is that test and not an approximation of it.
fn threshold(capacity: Option<(u16, u16)>, maze: &Maze) -> Option<TooSmall> {
    let Some((width, height)) = capacity else {
        return Some(TooSmall::LayoutFloor);
    };
    if maze.width() > width || maze.height() > height {
        return Some(TooSmall::MazeFit);
    }
    None
}
