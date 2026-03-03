// =============================================================================
// main.rs — Entry Point & Event Loop
// =============================================================================
//
// Sets up the terminal, runs the 60 FPS event loop, and restores the terminal
// on exit. The loop is simple:
//
//   1. Poll for keyboard events (non-blocking, ~16ms timeout for 60 FPS)
//   2. Handle input → update app state
//   3. Call app.update() for auto-rotation
//   4. Render the frame
//
// Terminal setup/teardown uses crossterm's raw mode and alternate screen.
// =============================================================================

mod app;
mod geometry;
mod projection;
mod rotation;
mod theme;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;

/// Frame duration for ~60 FPS (16.67ms)
const FRAME_DURATION: Duration = Duration::from_millis(16);

fn main() -> io::Result<()> {
    // === Panic Recovery ===
    // Install a panic hook that restores the terminal before printing the panic.
    // Without this, a panic leaves the terminal in raw mode and alternate screen.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    // === Terminal Setup ===
    // Enable raw mode so we get individual key presses without waiting for Enter
    enable_raw_mode()?;

    // Switch to the alternate screen buffer — this preserves the user's
    // scrollback when they quit the application
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // Create the ratatui terminal with crossterm backend
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Hide the cursor for a clean wireframe display
    terminal.hide_cursor()?;

    // === Run the Application ===
    let mut app = App::new();
    let result = run_loop(&mut terminal, &mut app);

    // === Terminal Teardown ===
    // Always restore the terminal, even if the app errored
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// The main event loop — runs until the user quits.
///
/// Each iteration:
/// 1. Renders the current frame
/// 2. Polls for input with a 16ms timeout (≈60 FPS)
/// 3. Handles any key events
/// 4. Updates auto-rotation state
fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    while app.running {
        // Render the UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Poll for events with a timeout matching our target frame rate.
        // This means we get at most ~60 frames per second, and the CPU
        // sleeps during the poll when there's nothing to do.
        if event::poll(FRAME_DURATION)? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events (not release or repeat)
                if key.kind == KeyEventKind::Press {
                    handle_key(app, key.code);
                }
            }
        }

        // Apply per-frame updates (auto-rotation, etc.)
        app.update();
    }

    Ok(())
}

/// Map key codes to application actions.
///
/// The control scheme:
///   0-9:        Select the active rotation plane
///   Left/Right: Rotate the active plane by ±θ
///   Space:      Toggle auto-rotation
///   q / Esc:    Quit the application
fn handle_key(app: &mut App, key: KeyCode) {
    match key {
        // Plane selection: digit keys 0 through 9
        KeyCode::Char('0') => app.select_plane(0),
        KeyCode::Char('1') => app.select_plane(1),
        KeyCode::Char('2') => app.select_plane(2),
        KeyCode::Char('3') => app.select_plane(3),
        KeyCode::Char('4') => app.select_plane(4),
        KeyCode::Char('5') => app.select_plane(5),
        KeyCode::Char('6') => app.select_plane(6),
        KeyCode::Char('7') => app.select_plane(7),
        KeyCode::Char('8') => app.select_plane(8),
        KeyCode::Char('9') => app.select_plane(9),

        // Rotation: left arrow = negative direction, right = positive
        KeyCode::Left => app.rotate_active(-1.0),
        KeyCode::Right => app.rotate_active(1.0),

        // Toggle auto-rotation with Space
        KeyCode::Char(' ') => app.toggle_auto_rotate(),

        // Quit with 'q' or Escape
        KeyCode::Char('q') => app.quit(),
        KeyCode::Esc => app.quit(),

        _ => {} // Ignore all other keys
    }
}
