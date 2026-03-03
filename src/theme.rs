// =============================================================================
// theme.rs — Centralized Style & Color System
// =============================================================================
//
// All visual styling is defined here. No hardcoded colors or styles exist
// anywhere else in the codebase. Components reference these constants/functions.
// =============================================================================

use ratatui::style::{Color, Modifier, Style};

// =============================================================================
// Color Palette — Cyberpunk-inspired neon on dark background
// =============================================================================

/// Background color for the entire TUI
pub const BG_COLOR: Color = Color::Rgb(10, 10, 18);

/// Primary text color (soft white)
pub const TEXT_PRIMARY: Color = Color::Rgb(200, 200, 210);

/// Dimmed/secondary text color
pub const TEXT_DIM: Color = Color::Rgb(90, 90, 110);

/// Accent color for the active/selected plane (bright cyan)
pub const ACCENT_ACTIVE: Color = Color::Rgb(0, 255, 200);

/// Inactive plane label color (muted teal)
pub const ACCENT_INACTIVE: Color = Color::Rgb(50, 80, 90);

/// Title/header color (electric blue)
pub const TITLE_COLOR: Color = Color::Rgb(80, 160, 255);

/// Border color for panels
pub const BORDER_COLOR: Color = Color::Rgb(40, 60, 80);

/// Key hint color
pub const KEY_HINT_COLOR: Color = Color::Rgb(120, 140, 160);

// =============================================================================
// Edge Depth Shading — Near edges are bright, far edges are dim
// =============================================================================

/// The brightest edge color (near the camera) — vivid green
const EDGE_NEAR: (u8, u8, u8) = (0, 255, 140);

/// The dimmest edge color (far from the camera) — dark blue-gray
const EDGE_FAR: (u8, u8, u8) = (30, 40, 70);

/// Interpolate edge color based on depth.
///
/// `depth` is expected to be in roughly [-1.0, 1.0] range (from vertex w+v average).
/// We normalize it to [0.0, 1.0] where 0.0 = far, 1.0 = near.
pub fn edge_color_from_depth(depth: f32) -> Color {
    // Clamp depth from [-1.0, 1.0] to [0.0, 1.0] for interpolation
    let t = ((depth + 1.0) / 2.0).clamp(0.0, 1.0);

    let r = lerp_u8(EDGE_FAR.0, EDGE_NEAR.0, t);
    let g = lerp_u8(EDGE_FAR.1, EDGE_NEAR.1, t);
    let b = lerp_u8(EDGE_FAR.2, EDGE_NEAR.2, t);

    Color::Rgb(r, g, b)
}

/// Linear interpolation between two u8 values.
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let result = a as f32 + (b as f32 - a as f32) * t;
    result.clamp(0.0, 255.0) as u8
}

// =============================================================================
// Pre-built Styles — Convenience functions for consistent styling
// =============================================================================

/// Style for the main title block
pub fn title_style() -> Style {
    Style::default()
        .fg(TITLE_COLOR)
        .add_modifier(Modifier::BOLD)
}

/// Style for the active rotation plane in the HUD
pub fn active_plane_style() -> Style {
    Style::default()
        .fg(ACCENT_ACTIVE)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

/// Style for inactive rotation planes in the HUD
pub fn inactive_plane_style() -> Style {
    Style::default().fg(ACCENT_INACTIVE)
}

/// Style for key hints at the bottom
pub fn key_hint_style() -> Style {
    Style::default().fg(KEY_HINT_COLOR)
}

/// Style for general borders
pub fn border_style() -> Style {
    Style::default().fg(BORDER_COLOR)
}

/// Style for angle readout values
pub fn angle_value_style() -> Style {
    Style::default().fg(TEXT_PRIMARY)
}

/// Style for dim/secondary text
pub fn dim_text_style() -> Style {
    Style::default().fg(TEXT_DIM)
}
