// =============================================================================
// app.rs — Application State & Input Handling
// =============================================================================
//
// The App struct owns all mutable state: the rotation angles, the active plane,
// and the auto-rotation toggle. It provides methods for input handling and
// per-frame updates.
//
// The event loop in main.rs calls App::handle_input() and App::update() each
// frame, keeping the control flow simple and testable.
// =============================================================================

use crate::rotation::{RotationPlane, RotationState, PLANE_COUNT};

/// The rotation speed in radians per key press
const ROTATION_SPEED: f32 = 0.05;

/// The auto-rotation speed in radians per frame (~0.02 rad at 60 FPS ≈ ~1.2 rad/s)
const AUTO_ROTATION_SPEED: f32 = 0.02;

/// Camera distances for the projection pipeline.
/// These control the "focal length" at each projection stage.
/// Larger values = less perspective distortion, smaller = more dramatic foreshortening.
pub const CAMERA_DISTANCE_5D: f32 = 5.0;
pub const CAMERA_DISTANCE_4D: f32 = 5.0;
pub const CAMERA_DISTANCE_3D: f32 = 5.0;

/// The rendering scale factor — multiplied against 2D projected coordinates
/// to map from the [-1,1] math space onto the terminal canvas.
pub const RENDER_SCALE: f32 = 1.8;

/// Core application state.
pub struct App {
    /// Current rotation angles for all 10 planes
    pub rotation: RotationState,
    /// Index of the currently selected rotation plane (0..9)
    pub active_plane: usize,
    /// Whether auto-rotation is enabled
    pub auto_rotate: bool,
    /// Whether the application is still running
    pub running: bool,
}

impl App {
    /// Create a new App with default state.
    pub fn new() -> Self {
        Self {
            rotation: RotationState::new(),
            active_plane: 0,
            auto_rotate: false,
            running: true,
        }
    }

    /// Get the currently selected RotationPlane enum variant.
    pub fn selected_plane(&self) -> RotationPlane {
        RotationPlane::ALL[self.active_plane]
    }

    /// Rotate the active plane by the given delta (positive = right, negative = left).
    pub fn rotate_active(&mut self, direction: f32) {
        let plane = self.selected_plane();
        self.rotation.rotate(plane, direction * ROTATION_SPEED);
    }

    /// Called once per frame — applies auto-rotation if enabled.
    ///
    /// Auto-rotation spins the active plane continuously for a mesmerizing effect.
    /// We also add a slow rotation on a secondary plane (WV) for extra visual depth.
    pub fn update(&mut self) {
        if self.auto_rotate {
            let plane = self.selected_plane();
            self.rotation.rotate(plane, AUTO_ROTATION_SPEED);

            // Add a slower secondary rotation for visual richness
            // Choose a plane that's different from the active one
            let secondary_idx = (self.active_plane + 5) % PLANE_COUNT;
            let secondary_plane = RotationPlane::ALL[secondary_idx];
            self.rotation
                .rotate(secondary_plane, AUTO_ROTATION_SPEED * 0.3);
        }
    }

    /// Select a rotation plane by its index (0-9). Clamps to valid range.
    pub fn select_plane(&mut self, index: usize) {
        if index < PLANE_COUNT {
            self.active_plane = index;
        }
    }

    /// Toggle auto-rotation on/off.
    pub fn toggle_auto_rotate(&mut self) {
        self.auto_rotate = !self.auto_rotate;
    }

    /// Signal the app to quit.
    pub fn quit(&mut self) {
        self.running = false;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Unit Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let app = App::new();
        assert_eq!(app.active_plane, 0);
        assert!(!app.auto_rotate);
        assert!(app.running);
        // All angles should be zero
        for angle in &app.rotation.angles {
            assert!(angle.abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_select_plane() {
        let mut app = App::new();
        app.select_plane(7);
        assert_eq!(app.active_plane, 7);
        // Out-of-range should be ignored
        app.select_plane(15);
        assert_eq!(app.active_plane, 7);
    }

    #[test]
    fn test_rotate_active() {
        let mut app = App::new();
        app.select_plane(3); // XV plane
        app.rotate_active(1.0);
        assert!((app.rotation.angles[3] - ROTATION_SPEED).abs() < 1e-6);
    }

    #[test]
    fn test_toggle_auto_rotate() {
        let mut app = App::new();
        assert!(!app.auto_rotate);
        app.toggle_auto_rotate();
        assert!(app.auto_rotate);
        app.toggle_auto_rotate();
        assert!(!app.auto_rotate);
    }

    #[test]
    fn test_update_with_auto_rotate() {
        let mut app = App::new();
        app.toggle_auto_rotate();
        app.update();
        // The active plane (0 = XY) should have rotated
        assert!(app.rotation.angles[0].abs() > 0.0);
    }

    #[test]
    fn test_quit() {
        let mut app = App::new();
        assert!(app.running);
        app.quit();
        assert!(!app.running);
    }
}
