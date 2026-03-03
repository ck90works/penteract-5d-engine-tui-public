# Architecture

## Overview
Penteract Engine is a terminal-based 5D hypercube (Penteract) wireframe visualizer built with Rust and ratatui.

## Module Map

```
src/
├── main.rs        — Entry point, event loop, terminal setup/teardown
├── app.rs         — Application state (rotation angles, active plane, auto-rotate)
├── geometry.rs    — Vertex & edge generation for the 32-vertex, 80-edge Penteract
├── projection.rs  — Cascading perspective pipeline: 5D → 4D → 3D → 2D + depth
├── rotation.rs    — 10 rotation planes, 5×5 rotation matrices, composite rotation
├── theme.rs       — Centralized color palette, depth gradient, and style system
└── ui.rs          — ratatui rendering: Canvas (wireframe) + HUD (sidebar)
```

## Rendering Pipeline

```
generate_vertices() → [32 × 5D vectors]
        │
  composite_rotation() → 5×5 matrix
        │
  project_vertex()     → (screen_x, screen_y, depth)
        │
  edge_color_from_depth(depth) → Color   (theme.rs)
        │
  Canvas::paint()      → ratatui terminal output
```

## Theme System (`theme.rs`)

All colors and styles are centralized — no hardcoded values elsewhere.

### Color Palette
A single fixed cyberpunk-inspired color scheme with neon-on-dark aesthetics.

| Constant | Value | Usage |
|----------|-------|-------|
| `BG_COLOR` | `(10, 10, 18)` | Terminal background |
| `ACCENT_ACTIVE` | `(0, 255, 200)` | Active/selected plane highlight |
| `EDGE_NEAR` | `(0, 255, 140)` | Nearest edges (vivid green) |
| `EDGE_FAR` | `(30, 40, 70)` | Farthest edges (dark blue-gray) |

### Key Functions
- `edge_color_from_depth(depth: f32) → Color` — Linearly interpolates between `EDGE_NEAR` and `EDGE_FAR` based on normalized depth
- `lerp_u8(a, b, t)` — Internal helper for per-channel linear interpolation
- Style constructors: `title_style()`, `active_plane_style()`, `inactive_plane_style()`, `key_hint_style()`, `border_style()`, `angle_value_style()`, `dim_text_style()`

## Data Flow
- **Immutable geometry**: vertices and edges are computed once via `LazyLock` statics in `ui.rs`
- **Mutable state**: `App` struct in `app.rs` owns rotation angles and UI state
- **Frame loop**: `main.rs` runs a 60 FPS loop calling `app.update()` + `ui::draw()`
