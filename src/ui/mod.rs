//! The rendering. This is the only part of the library that names the terminal.

pub mod help;
pub mod layout;
pub mod legend;
pub mod maze;
pub mod palette;
pub mod panel;
pub mod stats;

use ratatui::Frame;

use crate::app::{Activity, App, Phase, TooSmall};

/// Draws one frame of the screen of section 7.
///
/// The layout is the five bands of section 7.1, and it is the same at every
/// terminal size. The glyph set is selected here, once for the frame, and
/// every band that draws a glyph reads it from this one value, so no frame can
/// mix the two glyph sets.
///
/// **The two too-small panels of section 13 replace different things.** A
/// maze that does not fit the terminal gives the maze-fit panel in place of
/// the maze pane, and every other band keeps rendering. Below the layout floor
/// the whole screen is the layout-floor panel, and nothing else is drawn: not
/// the legend, and not the help overlay. `App` decides which threshold fired,
/// and `render` reads the answer from [`App::too_small`].
pub fn render(frame: &mut Frame, app: &App) {
    let screen = frame.area();
    if app.too_small == Some(TooSmall::LayoutFloor) {
        frame.render_widget(&panel::layout_floor(screen), screen);
        return;
    }
    let glyphs = if app.ascii {
        &palette::ASCII
    } else {
        &palette::UNICODE
    };
    let bands = layout::bands(screen);
    frame.render_widget(layout::status_line(app), bands.status);
    if app.too_small == Some(TooSmall::MazeFit) {
        frame.render_widget(&panel::maze_fit(&app.maze, screen), bands.maze);
    } else {
        maze::render(app, glyphs, bands.maze, frame.buffer_mut());
    }
    stats::render(app, bands.stats, frame.buffer_mut());
    legend::render(app, glyphs, bands.legend, frame.buffer_mut());
    frame.render_widget(help::row(bands.help.width), bands.help);
    if app.help_open {
        help::render_overlay(screen, bands.maze, frame.buffer_mut());
    }
}

/// The kind of run that the phase draws on the maze and in the band, or `None`
/// where the phase draws no run in progress.
///
/// It is [`Phase::activity`], with one addition: `Solved` holds a solver run
/// and has no activity, because nothing is left to advance, but the screen
/// still draws that run. `Ready` answers `None`. The generation run it can
/// hold is over.
const fn shown_activity(phase: Phase) -> Option<Activity> {
    match phase {
        Phase::Solved => Some(Activity::Solving),
        phase => phase.activity(),
    }
}
