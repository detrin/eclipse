/*
 * HEXAGONAL COORDINATE SYSTEM EXPLANATION
 * ========================================
 *
 * This game uses an Axial Coordinate System for hexagonal grids.
 * Axial coordinates use two values (q, r) to represent positions on a hex grid.
 *
 * COORDINATE SYSTEM BASICS:
 * -------------------------
 * - q: The "column" coordinate (moves along one diagonal axis)
 * - r: The "row" coordinate (moves along another diagonal axis)
 * - s: The third implicit coordinate, always equals -(q + r)
 *
 * This is called "cube coordinates" when all three (q, r, s) are used,
 * but we only store (q, r) since s can be derived.
 *
 * VISUAL REPRESENTATION (Flat-top hexagons):
 * ------------------------------------------
 *        (-1,-1)  (0,-1)  (1,-1)
 *           \      |      /
 *      (-1,0) - (0,0) - (1,0)
 *           /      |      \
 *        (-1,1)   (0,1)   (1,1)
 *
 * THE SIX NEIGHBORS:
 * ------------------
 * From any hex at (q, r), the six adjacent hexes are:
 *   1. (q+1, r)   - East
 *   2. (q+1, r-1) - Northeast
 *   3. (q, r-1)   - Northwest
 *   4. (q-1, r)   - West
 *   5. (q-1, r+1) - Southwest
 *   6. (q, r+1)   - Southeast
 *
 * DISTANCE CALCULATION:
 * ---------------------
 * The distance between two hexes is calculated using cube coordinates:
 *   distance = (|dq| + |dr| + |ds|) / 2
 * where:
 *   dq = q1 - q2
 *   dr = r1 - r2
 *   ds = (q1+r1) - (q2+r2) = s1 - s2
 *
 * This gives the minimum number of hex steps to move from one hex to another.
 *
 * PIXEL CONVERSION:
 * -----------------
 * To detect chain intersections geometrically, we convert hex coordinates
 * to 2D Cartesian (x, y) pixel coordinates using standard formulas for
 * pointy-topped hexagons:
 *
 *   x = size * (sqrt(3) * q + sqrt(3)/2 * r)
 *   y = size * (3/2 * r)
 *
 * This allows us to treat chains as line segments and use geometric
 * intersection algorithms to detect when chains cross.
 *
 * BOARD LAYOUT FOR ECLIPSE:
 * --------------------------
 * The game board is a custom hexagonal shape with 49 total hexes.
 * - Center hex: (0, 0)
 * - The board has 180-degree rotational symmetry around the center
 * - Dark player starts on the left side (negative q)
 * - Light player starts on the right side (positive q)
 *
 * BOARD SHAPE (49 hexes total):
 *
 *          (-1,-3) (0,-3) (1,-3) (2,-3) (3,-3) (4,-3)        [r=-3: 6 hexes]
 *       (-2,-2)(-1,-2) (0,-2) (1,-2) (2,-2) (3,-2) (4,-2)    [r=-2: 7 hexes]
 *    (-3,-1)(-2,-1)(-1,-1) (0,-1) (1,-1) (2,-1) (3,-1) (4,-1) [r=-1: 8 hexes]
 *       (-3,0) (-2,0) (-1,0) (0,0) (1,0) (2,0) (3,0)         [r=0:  7 hexes]
 *    (-4,1) (-3,1)(-2,1)(-1,1) (0,1) (1,1) (2,1) (3,1)       [r=1:  8 hexes]
 *       (-4,2)(-3,2)(-2,2)(-1,2) (0,2) (1,2) (2,2)           [r=2:  7 hexes]
 *          (-4,3)(-3,3)(-2,3)(-1,3) (0,3) (1,3)              [r=3:  6 hexes]
 *
 * STARTING POSITIONS (in axial coordinates q, r):
 *
 * Light (right side):
 * - Comet: (3, 0)
 * - Short Top (a): (4, -3) & (4, -2)
 * - Short Bottom (b): (2, 2) & (1, 3)
 * - Long Outer (c): (4, -1) & (3, 1)
 * - Long Middle (d): (3, -1) & (2, 1)
 * - Long Inner (e): (2, -1) & (1, 1)
 *
 * Dark (left side):
 * - Comet: (-3, 0)
 * - Short Top (f): (-1, -3) & (-2, -2)
 * - Short Bottom (g): (-4, 2) & (-4, 3)
 * - Long Outer (h): (-3, -1) & (-4, 1)
 * - Long Middle (i): (-2, -1) & (-3, 1)
 * - Long Inner (j): (-1, -1) & (-2, 1)
 */

use std::cmp::{max, min};
use serde::{Deserialize, Serialize};

/// Represents a hexagonal tile using Axial Coordinates.
///
/// The axial coordinate system uses two values (q, r) where:
/// - q represents the column (diagonal axis)
/// - r represents the row (another diagonal axis)
/// - The third coordinate s = -(q + r) is implicit
///
/// This system makes hex grid calculations efficient and intuitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hex {
    pub q: i32,
    pub r: i32,
}

impl Hex {
    /// Creates a new hex at coordinates (q, r)
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Returns the implicit s coordinate: s = -(q + r)
    ///
    /// Used for distance calculations and validation.
    pub fn s(&self) -> i32 {
        -(self.q + self.r)
    }

    /// Adds two hex coordinates together (vector addition)
    ///
    /// Useful for applying direction vectors to move between hexes.
    pub fn add(&self, other: Hex) -> Hex {
        Hex::new(self.q + other.q, self.r + other.r)
    }

    /// Returns all six adjacent hexes (neighbors)
    ///
    /// The neighbors are returned in clockwise order starting from East:
    /// East, Northeast, Northwest, West, Southwest, Southeast
    pub fn neighbors(&self) -> Vec<Hex> {
        vec![
            Hex::new(self.q + 1, self.r),     // East
            Hex::new(self.q + 1, self.r - 1), // Northeast
            Hex::new(self.q, self.r - 1),     // Northwest
            Hex::new(self.q - 1, self.r),     // West
            Hex::new(self.q - 1, self.r + 1), // Southwest
            Hex::new(self.q, self.r + 1),     // Southeast
        ]
    }

    /// Checks if two hexes are aligned along a single axis (q, r, or s)
    ///
    /// Two hexes are aligned if they share the same coordinate on any of the three axes.
    /// This is used to validate chain configurations:
    /// - Adjacent hexes (distance 1) always share exactly one axis
    /// - Long chains should NOT be aligned (all axes must differ)
    ///
    /// # Arguments
    /// * `other` - The other hex to compare with
    ///
    /// # Returns
    /// `true` if hexes share any axis coordinate, `false` if all three axes differ
    pub fn shares_axis(&self, other: &Hex) -> bool {
        self.q == other.q || self.r == other.r || self.s() == other.s()
    }

    /// Calculates the minimum number of hex steps between two hexes
    ///
    /// Uses the cube coordinate distance formula:
    /// distance = (|dq| + |dr| + |ds|) / 2
    ///
    /// This represents the shortest path on the hex grid.
    pub fn distance(&self, other: &Hex) -> i32 {
        let dq = (self.q - other.q).abs();
        let dr = (self.r - other.r).abs();
        let ds = ((self.q + self.r) - (other.q + other.r)).abs();
        (dq + dr + ds) / 2
    }

    /// Converts hex coordinates to 2D Cartesian (pixel) coordinates
    ///
    /// Uses the standard pointy-topped hexagon conversion:
    /// - x = size * (sqrt(3) * q + sqrt(3)/2 * r)
    /// - y = size * (3/2 * r)
    ///
    /// This is essential for geometric intersection tests between chains.
    /// The size parameter determines the hex radius (set to 1.0 for simplicity).
    pub fn to_pixel(&self) -> (f64, f64) {
        let size = 1.0;
        let x = size * (3.0f64.sqrt() * self.q as f64 + 3.0f64.sqrt() / 2.0 * self.r as f64);
        let y = size * (3.0 / 2.0 * self.r as f64);
        (x, y)
    }

    /// Returns all hexes within a given distance from this hex
    ///
    /// This generates a "ring" or "filled circle" of hexes centered on this position.
    /// Used to find valid movement destinations for satellites within chain length.
    ///
    /// # Arguments
    /// * `max_dist` - Maximum distance (inclusive) from center hex
    ///
    /// # Returns
    /// Vector of all hexes within max_dist steps (excluding the center hex itself)
    pub fn get_hexes_in_range(&self, max_dist: i32) -> Vec<Hex> {
        let mut hexes = Vec::new();

        // Iterate through all possible q values in range
        for q in -max_dist..=max_dist {
            // For each q, calculate the valid range of r values
            // This forms a "cube" in cube coordinate space
            for r in max(-max_dist, -q - max_dist)..=min(max_dist, -q + max_dist) {
                let hex = self.add(Hex::new(q, r));
                // Exclude the center hex itself
                if hex != *self {
                    hexes.push(hex);
                }
            }
        }

        hexes
    }

    /// Checks if this hex is within the board bounds
    ///
    /// A hex is valid if its distance from the origin (0,0) is at most `radius`.
    /// This creates a hexagonal board shape.
    ///
    /// # Arguments
    /// * `radius` - Maximum allowed distance from center
    ///
    /// # Returns
    /// `true` if the hex is within bounds, `false` otherwise
    pub fn is_within_radius(&self, radius: i32) -> bool {
        self.distance(&Hex::new(0, 0)) <= radius
    }

    /// Checks if this hex is on the Eclipse game board
    ///
    /// The Eclipse board is a custom 49-hex shape with specific valid positions.
    /// Valid ranges for each row (r value):
    /// - r=-3: q in [-1, 4]   (6 hexes)
    /// - r=-2: q in [-2, 4]   (7 hexes)
    /// - r=-1: q in [-3, 4]   (8 hexes)
    /// - r=0:  q in [-3, 3]   (7 hexes)
    /// - r=1:  q in [-4, 3]   (8 hexes)
    /// - r=2:  q in [-4, 2]   (7 hexes)
    /// - r=3:  q in [-4, 1]   (6 hexes)
    ///
    /// # Returns
    /// `true` if the hex is on the board, `false` otherwise
    pub fn is_on_board(&self) -> bool {
        match self.r {
            -3 => self.q >= -1 && self.q <= 4,
            -2 => self.q >= -2 && self.q <= 4,
            -1 => self.q >= -3 && self.q <= 4,
            0  => self.q >= -3 && self.q <= 3,
            1  => self.q >= -4 && self.q <= 3,
            2  => self.q >= -4 && self.q <= 2,
            3  => self.q >= -4 && self.q <= 1,
            _  => false,
        }
    }
}

// =============================================================================
// GEOMETRIC INTERSECTION UTILITIES
// =============================================================================

/// Determines if two line segments intersect
///
/// This is the core function for detecting chain crossings in the game.
/// Uses the CCW (Counter-Clockwise) orientation test.
///
/// # Algorithm
/// Two line segments (p1-p2) and (p3-p4) intersect if and only if:
/// - p1 and p2 are on opposite sides of the line through p3-p4, AND
/// - p3 and p4 are on opposite sides of the line through p1-p2
///
/// # Arguments
/// * `p1`, `p2` - Endpoints of the first line segment (in pixel coordinates)
/// * `p3`, `p4` - Endpoints of the second line segment (in pixel coordinates)
///
/// # Returns
/// `true` if the segments intersect (cross each other), `false` otherwise
pub fn segments_intersect(p1: (f64, f64), p2: (f64, f64), p3: (f64, f64), p4: (f64, f64)) -> bool {
    /// Helper function to determine counter-clockwise orientation
    ///
    /// Returns true if point c is on the left side of the line from a to b.
    /// This is computed using the cross product:
    ///   (b - a) × (c - a) = (b.x - a.x)(c.y - a.y) - (b.y - a.y)(c.x - a.x)
    ///
    /// Positive result = counter-clockwise (left turn)
    /// Negative result = clockwise (right turn)
    /// Zero = collinear points
    fn ccw(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> bool {
        (c.1 - a.1) * (b.0 - a.0) > (b.1 - a.1) * (c.0 - a.0)
    }

    // Check if p1 and p2 are on opposite sides of line p3-p4
    let ccw1 = ccw(p1, p3, p4);
    let ccw2 = ccw(p2, p3, p4);

    // Check if p3 and p4 are on opposite sides of line p1-p2
    let ccw3 = ccw(p1, p2, p3);
    let ccw4 = ccw(p1, p2, p4);

    // Segments intersect if both pairs are on opposite sides
    (ccw1 != ccw2) && (ccw3 != ccw4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_distance() {
        let origin = Hex::new(0, 0);
        let nearby = Hex::new(1, 0);
        let faraway = Hex::new(2, 2);

        assert_eq!(origin.distance(&origin), 0);
        assert_eq!(origin.distance(&nearby), 1);
        assert_eq!(origin.distance(&faraway), 4);
    }

    #[test]
    fn test_hex_neighbors() {
        let center = Hex::new(0, 0);
        let neighbors = center.neighbors();

        assert_eq!(neighbors.len(), 6);
        assert!(neighbors.contains(&Hex::new(1, 0)));
        assert!(neighbors.contains(&Hex::new(-1, 0)));
    }

    #[test]
    fn test_segments_intersect() {
        // Two segments that cross
        let p1 = (0.0, 0.0);
        let p2 = (2.0, 2.0);
        let p3 = (0.0, 2.0);
        let p4 = (2.0, 0.0);
        assert!(segments_intersect(p1, p2, p3, p4));

        // Two segments that don't cross
        let p5 = (0.0, 0.0);
        let p6 = (1.0, 0.0);
        let p7 = (0.0, 1.0);
        let p8 = (1.0, 1.0);
        assert!(!segments_intersect(p5, p6, p7, p8));
    }

    #[test]
    fn test_is_within_radius() {
        // Test hexes at various distances from origin
        let center = Hex::new(0, 0);
        assert!(center.is_within_radius(4));
        assert!(center.is_within_radius(0));

        // Hexes at distance 1
        let dist1 = Hex::new(1, 0);
        assert!(dist1.is_within_radius(4));
        assert!(dist1.is_within_radius(1));
        assert!(!dist1.is_within_radius(0));

        // Hexes at distance 4 (on the edge)
        let edge1 = Hex::new(0, -4);
        let edge2 = Hex::new(4, -4);
        let edge3 = Hex::new(-4, 0);
        let edge4 = Hex::new(4, 0);
        assert!(edge1.is_within_radius(4));
        assert!(edge2.is_within_radius(4));
        assert!(edge3.is_within_radius(4));
        assert!(edge4.is_within_radius(4));
        assert!(!edge1.is_within_radius(3));

        // Hex beyond radius 4
        let outside = Hex::new(5, 0);
        assert!(!outside.is_within_radius(4));
        assert!(outside.is_within_radius(5));
    }

    #[test]
    fn test_is_on_board() {
        // Test center
        assert!(Hex::new(0, 0).is_on_board());

        // Test valid hexes at each row
        assert!(Hex::new(-1, -3).is_on_board()); // r=-3 left edge
        assert!(Hex::new(4, -3).is_on_board());  // r=-3 right edge
        assert!(Hex::new(0, -3).is_on_board());  // r=-3 middle

        assert!(Hex::new(-3, 0).is_on_board());  // r=0 left edge
        assert!(Hex::new(3, 0).is_on_board());   // r=0 right edge

        assert!(Hex::new(-4, 3).is_on_board());  // r=3 left edge
        assert!(Hex::new(1, 3).is_on_board());   // r=3 right edge

        // Test invalid hexes (outside board)
        assert!(!Hex::new(-2, -3).is_on_board()); // r=-3, q too small
        assert!(!Hex::new(5, -3).is_on_board());  // r=-3, q too large
        assert!(!Hex::new(4, 0).is_on_board());   // r=0, q too large
        assert!(!Hex::new(-4, 0).is_on_board());  // r=0, q too small
        assert!(!Hex::new(2, 3).is_on_board());   // r=3, q too large
        assert!(!Hex::new(-5, 3).is_on_board());  // r=3, q too small

        // Test invalid rows
        assert!(!Hex::new(0, -4).is_on_board());  // r=-4 not on board
        assert!(!Hex::new(0, 4).is_on_board());   // r=4 not on board
        assert!(!Hex::new(0, 10).is_on_board());  // r=10 way off board
    }

    #[test]
    fn test_starting_positions() {
        // Test that all starting positions are on the board

        // Light side (right)
        assert!(Hex::new(3, 0).is_on_board());    // Comet
        assert!(Hex::new(4, -3).is_on_board());   // Short Top (a)
        assert!(Hex::new(4, -2).is_on_board());   // Short Top (a)
        assert!(Hex::new(2, 2).is_on_board());    // Short Bottom (b)
        assert!(Hex::new(1, 3).is_on_board());    // Short Bottom (b)
        assert!(Hex::new(4, -1).is_on_board());   // Long Outer (c)
        assert!(Hex::new(3, 1).is_on_board());    // Long Outer (c)
        assert!(Hex::new(3, -1).is_on_board());   // Long Middle (d)
        assert!(Hex::new(2, 1).is_on_board());    // Long Middle (d)
        assert!(Hex::new(2, -1).is_on_board());   // Long Inner (e)
        assert!(Hex::new(1, 1).is_on_board());    // Long Inner (e)

        // Dark side (left)
        assert!(Hex::new(-3, 0).is_on_board());   // Comet
        assert!(Hex::new(-1, -3).is_on_board());  // Short Top (f)
        assert!(Hex::new(-2, -2).is_on_board());  // Short Top (f)
        assert!(Hex::new(-4, 2).is_on_board());   // Short Bottom (g)
        assert!(Hex::new(-4, 3).is_on_board());   // Short Bottom (g)
        assert!(Hex::new(-3, -1).is_on_board());  // Long Outer (h)
        assert!(Hex::new(-4, 1).is_on_board());   // Long Outer (h)
        assert!(Hex::new(-2, -1).is_on_board());  // Long Middle (i)
        assert!(Hex::new(-3, 1).is_on_board());   // Long Middle (i)
        assert!(Hex::new(-1, -1).is_on_board());  // Long Inner (j)
        assert!(Hex::new(-2, 1).is_on_board());   // Long Inner (j)
    }

    #[test]
    fn test_shares_axis() {
        // Test hexes that share q axis
        let h1 = Hex::new(1, -1);
        let h2 = Hex::new(1, 1);
        assert!(h1.shares_axis(&h2), "Hexes with same q should share axis");

        // Test hexes that share r axis
        let h3 = Hex::new(0, 2);
        let h4 = Hex::new(3, 2);
        assert!(h3.shares_axis(&h4), "Hexes with same r should share axis");

        // Test hexes that share s axis
        // s = -(q + r), so if s1 = s2, then -(q1+r1) = -(q2+r2)
        let h5 = Hex::new(0, 0);   // s = 0
        let h6 = Hex::new(1, -1);  // s = 0
        assert!(h5.shares_axis(&h6), "Hexes with same s should share axis");

        // Test hexes that don't share any axis (diagonal)
        let h7 = Hex::new(2, -1);  // q=2, r=-1, s=-1
        let h8 = Hex::new(1, 1);   // q=1, r=1, s=-2
        assert!(!h7.shares_axis(&h8), "Diagonal hexes should not share any axis");

        // Test another diagonal pair
        let h9 = Hex::new(0, 0);   // q=0, r=0, s=0
        let h10 = Hex::new(1, 1);  // q=1, r=1, s=-2
        assert!(!h9.shares_axis(&h10), "Diagonal hexes should not share any axis");

        // Test same hex (all axes shared)
        let h11 = Hex::new(5, 3);
        assert!(h11.shares_axis(&h11), "A hex should share all axes with itself");
    }
}
