// =============================================================================
// geometry.rs — The 5D Penteract Vertex & Edge Definitions
// =============================================================================
//
// A Penteract (5-cube) has:
//   - 2^5 = 32 vertices, each with coordinates in {-1, +1}^5
//   - 80 edges, connecting vertex pairs that differ in exactly one coordinate
//
// We store vertex data as nalgebra SVector<f32, 5> for seamless matrix math.
// =============================================================================

use nalgebra::SVector;

/// The number of vertices in a 5-cube: 2^5
pub const VERTEX_COUNT: usize = 32;

/// The number of edges in a 5-cube: 5 * 2^4 = 80
/// (Each vertex has 5 neighbors, and 32*5/2 = 80 undirected edges.)
pub const EDGE_COUNT: usize = 80;

/// Generate all 32 vertices of the Penteract.
///
/// Each vertex is a 5D vector where every component is either -1.0 or +1.0.
/// We enumerate them by treating the vertex index (0..31) as a 5-bit number
/// and mapping bit j → coordinate: 0 → -1.0, 1 → +1.0.
///
/// # Example
/// Vertex 0  = 0b00000 → [-1, -1, -1, -1, -1]
/// Vertex 31 = 0b11111 → [+1, +1, +1, +1, +1]
pub fn generate_vertices() -> [SVector<f32, 5>; VERTEX_COUNT] {
    let mut vertices = [SVector::<f32, 5>::zeros(); VERTEX_COUNT];

    for i in 0..VERTEX_COUNT {
        for j in 0..5 {
            // Extract bit j from i: if set → +1.0, else → -1.0
            vertices[i][j] = if (i >> j) & 1 == 1 { 1.0 } else { -1.0 };
        }
    }

    vertices
}

/// Generate all 80 edges of the Penteract as index pairs.
///
/// Two vertices are connected by an edge if and only if they differ
/// in exactly one coordinate. We check this via XOR: two vertex indices
/// that differ in exactly one bit (i.e., XOR is a power of two) share
/// an edge. We only record pairs where `a < b` to avoid duplicates.
pub fn generate_edges() -> [(usize, usize); EDGE_COUNT] {
    let mut edges = [(0usize, 0usize); EDGE_COUNT];
    let mut count = 0;

    for a in 0..VERTEX_COUNT {
        for b in (a + 1)..VERTEX_COUNT {
            let xor = a ^ b;
            // XOR is a power of two iff exactly one bit differs
            if xor != 0 && (xor & (xor - 1)) == 0 {
                edges[count] = (a, b);
                count += 1;
            }
        }
    }

    // Sanity: we should have found exactly 80 edges
    debug_assert_eq!(count, EDGE_COUNT);
    edges
}

// =============================================================================
// Unit Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_count() {
        let vertices = generate_vertices();
        assert_eq!(vertices.len(), VERTEX_COUNT);
    }

    #[test]
    fn test_vertex_values_are_plus_minus_one() {
        let vertices = generate_vertices();
        for v in &vertices {
            for i in 0..5 {
                assert!(
                    v[i] == -1.0 || v[i] == 1.0,
                    "Vertex coordinate must be ±1.0, got {}",
                    v[i]
                );
            }
        }
    }

    #[test]
    fn test_vertices_are_unique() {
        let vertices = generate_vertices();
        for i in 0..VERTEX_COUNT {
            for j in (i + 1)..VERTEX_COUNT {
                assert_ne!(vertices[i], vertices[j], "Duplicate vertices at {} and {}", i, j);
            }
        }
    }

    #[test]
    fn test_edge_count() {
        let edges = generate_edges();
        assert_eq!(edges.len(), EDGE_COUNT);
    }

    #[test]
    fn test_edges_differ_by_one_coordinate() {
        let vertices = generate_vertices();
        let edges = generate_edges();

        for &(a, b) in &edges {
            let mut diff_count = 0;
            for i in 0..5 {
                if (vertices[a][i] - vertices[b][i]).abs() > f32::EPSILON {
                    diff_count += 1;
                }
            }
            assert_eq!(
                diff_count, 1,
                "Edge ({}, {}) should differ in exactly 1 coordinate, found {}",
                a, b, diff_count
            );
        }
    }

    #[test]
    fn test_edge_indices_in_range() {
        let edges = generate_edges();
        for &(a, b) in &edges {
            assert!(a < VERTEX_COUNT);
            assert!(b < VERTEX_COUNT);
            assert!(a < b, "Edges should be ordered a < b");
        }
    }
}
