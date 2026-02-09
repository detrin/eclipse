use crate::board::Hex;
use crate::states::ChainId;
use serde::{Deserialize, Serialize};

// =============================================================================
// MOVE DEFINITIONS
// =============================================================================

/// Represents a possible move in the game.
///
/// In Eclipse, players can either move their comet or move one satellite
/// of a chain (as long as the chain is not crossed/immobilized).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Move {
    /// Move a comet to an adjacent hex position
    ///
    /// The comet can only move to adjacent empty hexes and cannot
    /// cross opponent chains (but can jump over own chains).
    MoveComet(Hex),

    /// Move one satellite of a chain from old_pos to new_pos
    ///
    /// # Fields
    /// * `chain_id` - Identifier of the chain being moved
    /// * `old_pos` - Current position of the satellite being moved
    /// * `new_pos` - Target position for the satellite
    ///
    /// # Constraints
    /// - The new position must keep the chain within its maximum length
    /// - The chain must not be immobilized (crossed by opponent)
    /// - The new position must be empty and on the board
    MoveSatellite {
        chain_id: ChainId,
        old_pos: Hex,
        new_pos: Hex,
    },
}

impl Move {
    /// Returns a human-readable description of the move
    pub fn describe(&self) -> String {
        match self {
            Move::MoveComet(pos) => {
                format!("Move comet to ({}, {})", pos.q, pos.r)
            }
            Move::MoveSatellite { chain_id, old_pos, new_pos } => {
                format!(
                    "Move satellite of chain {:?} from ({}, {}) to ({}, {})",
                    chain_id, old_pos.q, old_pos.r, new_pos.q, new_pos.r
                )
            }
        }
    }

    /// Returns true if this is a comet move
    pub fn is_comet_move(&self) -> bool {
        matches!(self, Move::MoveComet(_))
    }

    /// Returns true if this is a satellite move
    pub fn is_satellite_move(&self) -> bool {
        matches!(self, Move::MoveSatellite { .. })
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Hex;

    #[test]
    fn test_move_describe_comet() {
        let mv = Move::MoveComet(Hex::new(1, 2));
        let desc = mv.describe();
        assert!(desc.contains("comet"));
        assert!(desc.contains("(1, 2)"));
    }

    #[test]
    fn test_move_describe_satellite() {
        let mv = Move::MoveSatellite {
            chain_id: ChainId(5),
            old_pos: Hex::new(0, 0),
            new_pos: Hex::new(1, 1),
        };
        let desc = mv.describe();
        assert!(desc.contains("satellite"));
        assert!(desc.contains("(0, 0)"));
        assert!(desc.contains("(1, 1)"));
    }

    #[test]
    fn test_is_comet_move() {
        let comet_move = Move::MoveComet(Hex::new(1, 2));
        assert!(comet_move.is_comet_move());
        assert!(!comet_move.is_satellite_move());
    }

    #[test]
    fn test_is_satellite_move() {
        let satellite_move = Move::MoveSatellite {
            chain_id: ChainId(5),
            old_pos: Hex::new(0, 0),
            new_pos: Hex::new(1, 1),
        };
        assert!(satellite_move.is_satellite_move());
        assert!(!satellite_move.is_comet_move());
    }
}
