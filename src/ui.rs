// =============================================================================
// ui.rs — TUI Rendering with ratatui
// =============================================================================
//
// The UI is split into two zones:
//   [LEFT]  A large Canvas widget rendering the 80-edge wireframe
//   [RIGHT] A narrow HUD sidebar showing the 10 rotation planes and controls
//
// The Canvas uses ratatui's built-in `Line` shape to draw each edge.
// Edge colors are interpolated based on depth (near = bright, far = dim).
// =============================================================================

use std::sync::LazyLock;

use nalgebra::SVector;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{
        Block, Borders, Paragraph,
        canvas::{Canvas, Line as CanvasLine},
    },
};

use crate::app::{App, CAMERA_DISTANCE_3D, CAMERA_DISTANCE_4D, CAMERA_DISTANCE_5D, RENDER_SCALE};
use crate::geometry::{EDGE_COUNT, VERTEX_COUNT, generate_edges, generate_vertices};
use crate::projection::project_vertex;
use crate::rotation::{RotationPlane, composite_rotation};
use crate::theme;

// =============================================================================
// Cached Geometry — Computed once, reused every frame
// =============================================================================

/// Pre-computed Penteract vertices (32 × 5D vectors). Computed once on first use.
static VERTICES: LazyLock<[SVector<f32, 5>; VERTEX_COUNT]> =
    LazyLock::new(generate_vertices);

/// Pre-computed Penteract edges (80 index pairs). Computed once on first use.
static EDGES: LazyLock<[(usize, usize); EDGE_COUNT]> =
    LazyLock::new(generate_edges);

/// Render the full UI for one frame.
///
/// This is called every frame (~60 FPS) from the main loop.
/// It projects all 32 vertices, draws all 80 edges, and renders the HUD.
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Split the screen: large canvas on the left, narrow HUD on the right
    let chunks = Layout::horizontal([
        Constraint::Min(20),      // Canvas takes all remaining space
        Constraint::Length(28),   // HUD sidebar fixed at 28 columns
    ])
    .split(area);

    // Draw the wireframe canvas
    draw_canvas(frame, app, chunks[0]);

    // Draw the HUD sidebar
    draw_hud(frame, app, chunks[1]);
}

/// Render the 5D wireframe projection onto the Canvas widget.
///
/// Steps:
/// 1. Build the composite rotation matrix from the current RotationState
/// 2. Transform all 32 pristine vertices through the rotation
/// 3. Project each rotated vertex down to 2D + depth
/// 4. Draw all 80 edges as colored lines on the Canvas
fn draw_canvas(frame: &mut Frame, app: &App, area: Rect) {
    // Use cached vertices and edges — no per-frame allocation
    let vertices = &*VERTICES;
    let edges = &*EDGES;

    // Build the composite 5×5 rotation matrix from accumulated angles
    let rotation_matrix = composite_rotation(&app.rotation);

    // Transform and project all 32 vertices
    // Each entry is (screen_x, screen_y, depth)
    let projected: Vec<(f64, f64, f32)> = vertices
        .iter()
        .map(|v| {
            // Apply the rotation to the pristine vertex
            let rotated: SVector<f32, 5> = rotation_matrix * v;
            // Project 5D → 2D with depth info
            let (x, y, depth) = project_vertex(
                &rotated,
                CAMERA_DISTANCE_5D,
                CAMERA_DISTANCE_4D,
                CAMERA_DISTANCE_3D,
            );
            // Scale to fill the canvas nicely
            (
                (x * RENDER_SCALE) as f64,
                (y * RENDER_SCALE) as f64,
                depth,
            )
        })
        .collect();

    // Determine the canvas coordinate bounds
    // We use a fixed range to keep the projection stable
    let bounds = 4.0;

    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border_style())
                .title(" Penteract ∎ 5D Projection ")
                .title_style(theme::title_style())
                .style(Style::default().bg(theme::BG_COLOR)),
        )
        .x_bounds([-bounds, bounds])
        .y_bounds([-bounds, bounds])
        .paint(move |ctx| {
            // Draw each of the 80 edges as a colored line
            for &(a, b) in edges {
                let (x1, y1, d1) = projected[a];
                let (x2, y2, d2) = projected[b];

                // Average depth of both endpoints for edge color
                let avg_depth = (d1 + d2) / 2.0;
                let color = theme::edge_color_from_depth(avg_depth);

                ctx.draw(&CanvasLine {
                    x1,
                    y1,
                    x2,
                    y2,
                    color,
                });
            }
        });

    frame.render_widget(canvas, area);
}

/// Render the HUD sidebar with rotation plane list and controls.
///
/// Shows:
/// - Title
/// - All 10 rotation planes with the active one highlighted
/// - Current angle value for each plane
/// - Auto-rotation status
/// - Key bindings
fn draw_hud(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // Title
    lines.push(Line::from(Span::styled(
        "╔══ ROTATION PLANES ══╗",
        theme::title_style(),
    )));
    lines.push(Line::from(""));

    // List all 10 rotation planes
    for (idx, &plane) in RotationPlane::ALL.iter().enumerate() {
        let angle_deg = app.rotation.angles[idx].to_degrees();
        let is_active = idx == app.active_plane;

        let marker = if is_active { "▸ " } else { "  " };
        let key = format!("[{}]", idx);
        let label = format!(" {:<3}", plane.label());
        let angle_str = format!("{:>7.1}°", angle_deg);

        let style = if is_active {
            theme::active_plane_style()
        } else {
            theme::inactive_plane_style()
        };

        lines.push(Line::from(vec![
            Span::styled(marker, style),
            Span::styled(key, style),
            Span::styled(label, style),
            Span::styled(angle_str, theme::angle_value_style()),
        ]));
    }

    // Spacer
    lines.push(Line::from(""));

    // Auto-rotation status
    let auto_status = if app.auto_rotate {
        Span::styled("● AUTO-ROTATE: ON ", theme::active_plane_style())
    } else {
        Span::styled("○ Auto-rotate: off", theme::dim_text_style())
    };
    lines.push(Line::from(auto_status));

    // Spacer
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "╔══ CONTROLS ═════════╗",
        theme::title_style(),
    )));

    lines.push(Line::from(""));

    // Key hints
    let hints = [
        ("0-9", "Select plane"),
        ("←/→", "Rotate"),
        ("Space", "Auto-rotate"),
        ("q/Esc", "Quit"),
    ];

    for (key, desc) in &hints {
        lines.push(Line::from(vec![
            Span::styled(format!(" {:<7}", key), theme::active_plane_style()),
            Span::styled(*desc, theme::key_hint_style()),
        ]));
    }

    let hud = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .title(" HUD ")
            .title_style(theme::title_style())
            .style(Style::default().bg(theme::BG_COLOR)),
    );

    frame.render_widget(hud, area);
}
