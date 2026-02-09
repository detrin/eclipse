use crate::moves::Move;
use crate::states::GameState;

// =============================================================================
// BOT TRAIT
// =============================================================================

/// Trait for AI players (bots) that can choose moves in the game.
///
/// Implementations of this trait represent different bot strategies,
/// from simple random selection to sophisticated evaluation algorithms.
pub trait Bot {
    /// Chooses a move from the current game state.
    ///
    /// # Arguments
    /// * `game` - The current game state
    ///
    /// # Returns
    /// * `Some(Move)` - A chosen move if legal moves are available
    /// * `None` - If no legal moves are available
    fn choose_move(&self, game: &GameState) -> Option<Move>;

    /// Returns the name/description of this bot implementation
    fn name(&self) -> &str;
}
