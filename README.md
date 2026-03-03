# Penteract 5D Projection Engine 🌌

A real-time, terminal-based 5-dimensional hypercube (Penteract) wireframe renderer written in Rust. 

Projecting a 5D object down to 4D, then to 3D, and finally rasterizing it onto a 2D terminal grid in real-time is an absolute mind-bender. This project utilizes custom 5×5 Givens rotation matrices and a cascading perspective projection pipeline to visualize geometries that exist beyond human intuition.

## ✨ Features

- **Mathematical Accuracy:** Computes explicit 10-plane 5×5 rotation matrices using `nalgebra`.
- **Perspective Pipeline:** Cascading 5D $\to$ 4D $\to$ 3D $\to$ 2D rendering pipeline with depth mapping.
- **Floating-Point Stability:** Matrix generation eliminates rotational drift.
- **TUI Interface:** A beautiful 60 FPS terminal UI using `ratatui` with depth-based color shading (near edges are bright, far edges are dim).

---

## 🚀 Setup Guide

### Dependencies

To run the Penteract Engine, you must have the **Rust toolchain** installed. You can install Rust via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Installation

Clone this repository and navigate to the project directory:

```bash
git clone https://github.com/ck90works/penteract-5d-engine-tui-public.git
cd penteract-5d-engine-tui-public
```

### Running the Engine

Compile and run the project using `cargo`:

```bash
# Run in release mode for maximum performance (recommended)
cargo run --release
```

---

## 🕹️ Usage & Controls

The engine runs directly inside your terminal window. The UI is split between a live rendering canvas on the left and a HUD displaying the 10 rotation planes on the right.

| Key / Input | Action |
| --- | --- |
| **`0` - `9`** | Select the Active Rotation Plane (e.g., `0` for XY, `9` for WV) |
| **`Left Arrow`** | Rotate the selected plane by $-\theta$ |
| **`Right Arrow`** | Rotate the selected plane by $+\theta$ |
| **`Space`** | Toggle Auto-Rotation (hypnotic continuous movement) |
| **`q`** or **`Esc`** | Quit the application |

---

## 📊 Codebase Metrics

The engine is built with clean, testable architecture. Below is the total count of Source Lines of Code (SLOC), excluding blank lines and comments.

| Type | Count |
| --- | --- |
| Code | 481 Lines |
| Test | 234 Lines |

---

## 🧠 The Mathematics: 10 Planes of Rotation

Human brains are hardwired for 3D. We can somewhat comprehend 4D—think of a Tesseract, the 4-dimensional equivalent of a cube. But a 5-dimensional object—like a **Penteract** (a 5-cube)—requires $\binom{5}{2} = 10$ different planes of rotation: 

$XY, XZ, XW, XV, YZ, YW, YV, ZW, ZV, WV$

This project proves that Rust's zero-cost abstractions make it the perfect tool to grapple with the same kinds of higher-dimensional spaces studied in theoretical physics and machine learning dimensional reduction spaces.

Enjoy the mathematical sandbox!