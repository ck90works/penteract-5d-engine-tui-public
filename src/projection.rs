// =============================================================================
// projection.rs — Cascading Perspective Projection Pipeline
// =============================================================================
//
// The pipeline collapses dimensions one at a time via perspective division:
//
//   5D → 4D:  Divide (x, y, z, w) by (distance - v)
//   4D → 3D:  Divide (x, y, z)    by (distance - w)
//   3D → 2D:  Divide (x, y)       by (distance - z)
//
// Each step simulates a "camera" at some distance along the highest remaining
// axis, looking back toward the origin. The perspective division creates the
// foreshortening effect that lets us perceive depth.
//
// We also return a depth value for shading: the average of the w and v
// components after rotation, used to dim faraway edges.
// =============================================================================

use nalgebra::SVector;

/// Minimum denominator for perspective division.
/// Prevents division-by-zero when a vertex coordinate approaches the camera distance.
const MIN_PERSPECTIVE_DENOM: f32 = 0.01;

/// Perspective-project a 5D point down to 4D.
///
/// We treat the 5th coordinate (v, index 4) as the "depth" axis.
/// The camera sits at `distance` along v, looking toward the origin.
/// The projection factor is `distance / (distance - v)`.
pub fn project_5d_to_4d(v: &SVector<f32, 5>, distance: f32) -> SVector<f32, 4> {
    // Clamp denominator to prevent division-by-zero singularity
    let denom = (distance - v[4]).abs().max(MIN_PERSPECTIVE_DENOM)
        * (distance - v[4]).signum();
    let w = distance / denom;
    SVector::<f32, 4>::new(v[0] * w, v[1] * w, v[2] * w, v[3] * w)
}

/// Perspective-project a 4D point down to 3D.
///
/// Same principle: the 4th coordinate (w, index 3) is the depth axis.
pub fn project_4d_to_3d(v: &SVector<f32, 4>, distance: f32) -> SVector<f32, 3> {
    // Clamp denominator to prevent division-by-zero singularity
    let denom = (distance - v[3]).abs().max(MIN_PERSPECTIVE_DENOM)
        * (distance - v[3]).signum();
    let w = distance / denom;
    SVector::<f32, 3>::new(v[0] * w, v[1] * w, v[2] * w)
}

/// Perspective-project a 3D point down to 2D screen coordinates.
///
/// Standard 3D→2D: the z coordinate (index 2) is the depth axis.
/// Returns (screen_x, screen_y).
pub fn project_3d_to_2d(v: &SVector<f32, 3>, distance: f32) -> (f32, f32) {
    // Clamp denominator to prevent division-by-zero singularity
    let denom = (distance - v[2]).abs().max(MIN_PERSPECTIVE_DENOM)
        * (distance - v[2]).signum();
    let w = distance / denom;
    (v[0] * w, v[1] * w)
}

/// Full projection pipeline: 5D → 2D with depth information.
///
/// # Arguments
/// * `vertex` — The rotated 5D vertex
/// * `d5` — Camera distance for 5D→4D projection
/// * `d4` — Camera distance for 4D→3D projection
/// * `d3` — Camera distance for 3D→2D projection
///
/// # Returns
/// `(screen_x, screen_y, depth)` where depth is the average of the
/// original v and w coordinates, useful for depth-based shading.
pub fn project_vertex(
    vertex: &SVector<f32, 5>,
    d5: f32,
    d4: f32,
    d3: f32,
) -> (f32, f32, f32) {
    // Save the higher-dimensional coordinates for depth shading
    // before they are projected away
    let depth = (vertex[3] + vertex[4]) / 2.0;

    // Cascade the projections: 5D → 4D → 3D → 2D
    let v4 = project_5d_to_4d(vertex, d5);
    let v3 = project_4d_to_3d(&v4, d4);
    let (x, y) = project_3d_to_2d(&v3, d3);

    (x, y, depth)
}

// =============================================================================
// Unit Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::SVector;

    #[test]
    fn test_origin_projects_to_origin() {
        // A point at the origin in 5D should project to (0, 0) in 2D
        let origin = SVector::<f32, 5>::zeros();
        let (x, y, _depth) = project_vertex(&origin, 5.0, 5.0, 5.0);
        assert!(x.abs() < 1e-6, "x should be ~0, got {}", x);
        assert!(y.abs() < 1e-6, "y should be ~0, got {}", y);
    }

    #[test]
    fn test_symmetric_points_project_symmetrically() {
        // Two points that are mirror images across the x-axis should project
        // to mirror-image 2D coordinates
        let p1 = SVector::<f32, 5>::new(1.0, 0.5, 0.0, 0.0, 0.0);
        let p2 = SVector::<f32, 5>::new(-1.0, 0.5, 0.0, 0.0, 0.0);

        let (x1, y1, _) = project_vertex(&p1, 5.0, 5.0, 5.0);
        let (x2, y2, _) = project_vertex(&p2, 5.0, 5.0, 5.0);

        assert!((x1 + x2).abs() < 1e-6, "x coordinates should be negatives");
        assert!((y1 - y2).abs() < 1e-6, "y coordinates should be equal");
    }

    #[test]
    fn test_known_projection() {
        // Hand-verify: a point at [1, 0, 0, 0, 0] with all distances = 5.0
        // 5D→4D: factor = 5/(5-0) = 1.0 → [1, 0, 0, 0]
        // 4D→3D: factor = 5/(5-0) = 1.0 → [1, 0, 0]
        // 3D→2D: factor = 5/(5-0) = 1.0 → (1.0, 0.0)
        let p = SVector::<f32, 5>::new(1.0, 0.0, 0.0, 0.0, 0.0);
        let (x, y, depth) = project_vertex(&p, 5.0, 5.0, 5.0);
        assert!((x - 1.0).abs() < 1e-6, "Expected x=1.0, got {}", x);
        assert!(y.abs() < 1e-6, "Expected y=0.0, got {}", y);
        assert!(depth.abs() < 1e-6, "Expected depth=0.0, got {}", depth);
    }

    #[test]
    fn test_depth_calculation() {
        // Depth should be average of w (index 3) and v (index 4)
        let p = SVector::<f32, 5>::new(0.0, 0.0, 0.0, 0.8, -0.4);
        let (_, _, depth) = project_vertex(&p, 5.0, 5.0, 5.0);
        let expected_depth = (0.8 + (-0.4)) / 2.0; // 0.2
        assert!(
            (depth - expected_depth).abs() < 1e-6,
            "Expected depth={}, got {}",
            expected_depth,
            depth
        );
    }

    #[test]
    fn test_5d_to_4d_perspective() {
        // Point with v=1.0, distance=5.0 → factor = 5/(5-1) = 1.25
        let p = SVector::<f32, 5>::new(2.0, 3.0, 4.0, 1.0, 1.0);
        let result = project_5d_to_4d(&p, 5.0);
        let factor = 5.0 / (5.0 - 1.0); // 1.25
        assert!((result[0] - 2.0 * factor).abs() < 1e-5);
        assert!((result[1] - 3.0 * factor).abs() < 1e-5);
        assert!((result[2] - 4.0 * factor).abs() < 1e-5);
        assert!((result[3] - 1.0 * factor).abs() < 1e-5);
    }
}
