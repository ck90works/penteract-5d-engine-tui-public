// =============================================================================
// rotation.rs — 5D Rotation Matrices for all 10 Planes
// =============================================================================
//
// In N-dimensional space, rotations occur in planes, not around axes.
// For ℝ⁵, the number of orthogonal planes is C(5,2) = 10:
//   XY, XZ, XW, XV, YZ, YW, YV, ZW, ZV, WV
//
// We represent rotation state as 10 angles and build a composite 5×5
// rotation matrix by multiplying all 10 individual plane-rotation matrices.
//
// CRITICAL DESIGN: We never mutate the original vertices. Each frame we
// rebuild the full rotation matrix from the accumulated angles and apply
// it fresh to the pristine base vertices. This prevents floating-point drift.
// =============================================================================

use nalgebra::SMatrix;
use std::fmt;

/// The total number of rotation planes in ℝ⁵: C(5,2) = 10
pub const PLANE_COUNT: usize = 10;

/// Enumerates all 10 orthogonal rotation planes in 5D space.
///
/// Each variant stores the two axis indices (0-4) that define the plane.
/// Convention: X=0, Y=1, Z=2, W=3, V=4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationPlane {
    XY = 0,
    XZ = 1,
    XW = 2,
    XV = 3,
    YZ = 4,
    YW = 5,
    YV = 6,
    ZW = 7,
    ZV = 8,
    WV = 9,
}

impl RotationPlane {
    /// Returns all 10 planes in canonical order.
    pub const ALL: [RotationPlane; PLANE_COUNT] = [
        RotationPlane::XY,
        RotationPlane::XZ,
        RotationPlane::XW,
        RotationPlane::XV,
        RotationPlane::YZ,
        RotationPlane::YW,
        RotationPlane::YV,
        RotationPlane::ZW,
        RotationPlane::ZV,
        RotationPlane::WV,
    ];

    /// Returns the two axis indices (i, j) that define this rotation plane.
    /// The rotation matrix will have cos/sin terms at rows/cols (i, j).
    pub fn axis_indices(self) -> (usize, usize) {
        match self {
            RotationPlane::XY => (0, 1),
            RotationPlane::XZ => (0, 2),
            RotationPlane::XW => (0, 3),
            RotationPlane::XV => (0, 4),
            RotationPlane::YZ => (1, 2),
            RotationPlane::YW => (1, 3),
            RotationPlane::YV => (1, 4),
            RotationPlane::ZW => (2, 3),
            RotationPlane::ZV => (2, 4),
            RotationPlane::WV => (3, 4),
        }
    }

    /// Human-readable label string.
    pub fn label(self) -> &'static str {
        match self {
            RotationPlane::XY => "XY",
            RotationPlane::XZ => "XZ",
            RotationPlane::XW => "XW",
            RotationPlane::XV => "XV",
            RotationPlane::YZ => "YZ",
            RotationPlane::YW => "YW",
            RotationPlane::YV => "YV",
            RotationPlane::ZW => "ZW",
            RotationPlane::ZV => "ZV",
            RotationPlane::WV => "WV",
        }
    }
}

impl fmt::Display for RotationPlane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Holds the 10 rotation angles (in radians) — one per plane.
///
/// This is the single source of truth for the current orientation.
/// The composite rotation matrix is rebuilt from these every frame.
#[derive(Debug, Clone)]
pub struct RotationState {
    /// Rotation angles in radians for each of the 10 planes
    pub angles: [f32; PLANE_COUNT],
}

impl RotationState {
    /// Create a new rotation state with all angles at zero (identity orientation).
    pub fn new() -> Self {
        Self {
            angles: [0.0; PLANE_COUNT],
        }
    }

    /// Increment the rotation angle for a specific plane by `delta` radians.
    ///
    /// Angles are wrapped to [-τ, τ] to prevent precision loss in sin/cos
    /// when values grow very large during extended auto-rotation.
    pub fn rotate(&mut self, plane: RotationPlane, delta: f32) {
        self.angles[plane as usize] += delta;
        // Wrap to prevent unbounded growth — keeps sin/cos accurate
        self.angles[plane as usize] %= std::f32::consts::TAU;
    }
}

impl Default for RotationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a 5×5 rotation matrix for a single plane at the given angle.
///
/// A rotation in the (i, j) plane is the identity matrix with four modified entries:
///   R[i][i] =  cos(θ)
///   R[j][j] =  cos(θ)
///   R[i][j] = -sin(θ)
///   R[j][i] =  sin(θ)
///
/// All other entries remain as the 5×5 identity.
pub fn rotation_matrix_5d(plane: RotationPlane, angle: f32) -> SMatrix<f32, 5, 5> {
    let mut mat = SMatrix::<f32, 5, 5>::identity();
    let (i, j) = plane.axis_indices();

    let cos_a = angle.cos();
    let sin_a = angle.sin();

    // Set the four rotation entries in the (i, j) sub-block
    mat[(i, i)] = cos_a;
    mat[(j, j)] = cos_a;
    mat[(i, j)] = -sin_a;
    mat[(j, i)] = sin_a;

    mat
}

/// Build the composite 5×5 rotation matrix by multiplying all 10 plane rotations.
///
/// The order of multiplication matters but is consistent frame-to-frame since we
/// always apply them in the canonical order (XY, XZ, ..., WV). Because we rebuild
/// from scratch each frame using the accumulated angles, there is no drift.
pub fn composite_rotation(state: &RotationState) -> SMatrix<f32, 5, 5> {
    let mut result = SMatrix::<f32, 5, 5>::identity();

    for (idx, &plane) in RotationPlane::ALL.iter().enumerate() {
        let angle = state.angles[idx];
        // Skip identity rotations (zero angle) for a minor optimization
        if angle.abs() > f32::EPSILON {
            result = rotation_matrix_5d(plane, angle) * result;
        }
    }

    result
}

// =============================================================================
// Unit Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::SMatrix;

    /// Helper: check if a matrix is approximately orthogonal (R * Rᵀ ≈ I).
    fn is_orthogonal(m: &SMatrix<f32, 5, 5>, tolerance: f32) -> bool {
        let product = m * m.transpose();
        let identity = SMatrix::<f32, 5, 5>::identity();
        (product - identity).norm() < tolerance
    }

    #[test]
    fn test_rotation_matrix_is_orthogonal() {
        // Every rotation matrix should be orthogonal for any angle
        for plane in RotationPlane::ALL {
            for &angle in &[0.0, 0.5, 1.0, std::f32::consts::PI, 3.7] {
                let mat = rotation_matrix_5d(plane, angle);
                assert!(
                    is_orthogonal(&mat, 1e-5),
                    "Rotation matrix for {} at angle {} is not orthogonal",
                    plane,
                    angle
                );
            }
        }
    }

    #[test]
    fn test_identity_rotation() {
        // A zero-angle rotation should return the identity matrix
        for plane in RotationPlane::ALL {
            let mat = rotation_matrix_5d(plane, 0.0);
            let identity = SMatrix::<f32, 5, 5>::identity();
            assert!(
                (mat - identity).norm() < 1e-6,
                "Zero-angle rotation for {} should be identity",
                plane
            );
        }
    }

    #[test]
    fn test_composite_identity() {
        // All-zero RotationState should produce the identity matrix
        let state = RotationState::new();
        let mat = composite_rotation(&state);
        let identity = SMatrix::<f32, 5, 5>::identity();
        assert!(
            (mat - identity).norm() < 1e-6,
            "Composite of zero rotations should be identity"
        );
    }

    #[test]
    fn test_composite_is_orthogonal() {
        // A composite rotation with arbitrary angles should still be orthogonal
        let mut state = RotationState::new();
        state.angles = [0.1, 0.3, -0.5, 1.2, 0.7, -0.9, 0.4, 2.1, -1.5, 0.8];
        let mat = composite_rotation(&state);
        assert!(
            is_orthogonal(&mat, 1e-4),
            "Composite rotation matrix should be orthogonal"
        );
    }

    #[test]
    fn test_rotation_state_rotate() {
        let mut state = RotationState::new();
        state.rotate(RotationPlane::XY, 0.5);
        assert!((state.angles[0] - 0.5).abs() < f32::EPSILON);
        state.rotate(RotationPlane::XY, 0.3);
        assert!((state.angles[0] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_plane_count() {
        assert_eq!(RotationPlane::ALL.len(), PLANE_COUNT);
    }
}
