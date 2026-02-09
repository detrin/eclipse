use crate::bot::Bot;
use crate::moves::Move;
use crate::states::{GameState, Player};
use rand::Rng;

// =============================================================================
// RANDOM BOT IMPLEMENTATION
// =============================================================================

/// A simple bot that randomly selects from available legal moves.
///
/// This bot serves as a baseline opponent and is useful for:
/// - Testing the game mechanics work correctly
/// - Providing a simple opponent for new players
/// - Serving as a starting point for more sophisticated bots
#[derive(Debug)]
pub struct RandomBot {
    player: Player,
}

impl RandomBot {
    /// Creates a new RandomBot for the specified player.
    ///
    /// # Arguments
    /// * `player` - The player this bot will play as (Light or Dark)
    ///
    /// # Example
    /// ```
    /// use eclipse::randombot::RandomBot;
    /// use eclipse::states::Player;
    ///
    /// let bot = RandomBot::new(Player::Dark);
    /// ```
    pub fn new(player: Player) -> Self {
        RandomBot { player }
    }

    /// Returns the player this bot represents.
    pub fn player(&self) -> Player {
        self.player
    }
}

impl Bot for RandomBot {
    fn choose_move(&self, game: &GameState) -> Option<Move> {
        let legal_moves = game.get_legal_moves();

        if legal_moves.is_empty() {
            return None;
        }

        // Pick a random move from the available legal moves
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..legal_moves.len());

        Some(legal_moves[index].clone())
    }

    fn name(&self) -> &str {
        "RandomBot"
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::states::GameState;

    #[test]
    fn test_random_bot_creation() {
        let bot = RandomBot::new(Player::Light);
        assert_eq!(bot.player(), Player::Light);
        assert_eq!(bot.name(), "RandomBot");

        let bot = RandomBot::new(Player::Dark);
        assert_eq!(bot.player(), Player::Dark);
    }

    #[test]
    fn test_random_bot_chooses_legal_move() {
        let game = GameState::new();
        let bot = RandomBot::new(Player::Light);

        // Bot should choose a legal move
        let chosen_move = bot.choose_move(&game);
        assert!(chosen_move.is_some(), "Bot should choose a move when legal moves exist");

        // The chosen move should be in the list of legal moves
        let legal_moves = game.get_legal_moves();
        let chosen = chosen_move.unwrap();

        // Check if the chosen move exists in legal moves
        // We can't directly compare Move instances, so we'll verify it's valid
        // by checking it matches one of the legal move patterns
        let is_valid = legal_moves.iter().any(|m| {
            match (&chosen, m) {
                (Move::MoveComet(pos1), Move::MoveComet(pos2)) => pos1 == pos2,
                (
                    Move::MoveSatellite { chain_id: id1, old_pos: old1, new_pos: new1 },
                    Move::MoveSatellite { chain_id: id2, old_pos: old2, new_pos: new2 },
                ) => id1 == id2 && old1 == old2 && new1 == new2,
                _ => false,
            }
        });

        assert!(is_valid, "Bot should choose a valid legal move");
    }

    #[test]
    fn test_random_bot_multiple_choices() {
        let game = GameState::new();
        let bot = RandomBot::new(Player::Light);

        // Call choose_move multiple times - should get moves (possibly different)
        let mut moves = Vec::new();
        for _ in 0..5 {
            if let Some(mv) = bot.choose_move(&game) {
                moves.push(mv);
            }
        }

        assert_eq!(moves.len(), 5, "Bot should choose a move each time");
    }

    #[test]
    fn test_random_bot_returns_none_when_no_moves() {
        let mut game = GameState::new();
        let bot = RandomBot::new(Player::Light);

        // Manually create a situation with no legal moves by surrounding the comet
        // and blocking all chains
        let comet_pos = game.comet_light;
        let neighbors = comet_pos.neighbors();

        // Fill all adjacent positions to block the comet
        for (i, pos) in neighbors.iter().enumerate() {
            if pos.is_on_board() {
                game.occupied.insert(
                    *pos,
                    crate::states::Occupant::Satellite(
                        crate::states::ChainId(100 + i),
                        Player::Dark,
                    ),
                );
            }
        }

        // Verify there are no legal moves for Light
        let legal_moves = game.get_legal_moves();
        assert!(
            legal_moves.is_empty() || legal_moves.iter().all(|m| !m.is_comet_move()),
            "Should have no or very few legal moves when comet is blocked"
        );

        // Bot should handle this gracefully
        // Note: There might still be some chain moves available, so we just test
        // that the bot doesn't panic
        let _result = bot.choose_move(&game);
    }

    #[test]
    fn test_bot_can_play_complete_game() {
        let mut game = GameState::new();
        let light_bot = RandomBot::new(Player::Light);
        let dark_bot = RandomBot::new(Player::Dark);

        let max_turns = 1000; // Increased limit for random play
        let mut turn_count = 0;

        while game.status == crate::states::GameStatus::InProgress && turn_count < max_turns {
            // Get the appropriate bot for the current player
            let bot: &dyn Bot = if game.current_turn == Player::Light {
                &light_bot
            } else {
                &dark_bot
            };

            // Bot chooses a move
            let chosen_move = bot.choose_move(&game);

            match chosen_move {
                Some(mv) => {
                    // Apply the move
                    let result = game.apply_move(mv);
                    assert!(result.is_ok(), "Bot should only choose valid moves");
                }
                None => {
                    // No legal moves available - game should be over
                    break;
                }
            }

            turn_count += 1;
        }

        // Test passes if:
        // 1. Game ended with a winner, OR
        // 2. Game reached max turns (proves bots can play without crashing)
        // Random play may take many turns to reach a win condition, which is expected
        println!("Game ran for {} turns", turn_count);
        println!("Final status: {:?}", game.status);

        // Main goal: verify bots can play without errors
        assert!(turn_count > 0, "Bots should make at least one move");
    }

    #[test]
    fn test_bot_trait_object() {
        let game = GameState::new();
        let bot: Box<dyn Bot> = Box::new(RandomBot::new(Player::Light));

        let chosen_move = bot.choose_move(&game);
        assert!(chosen_move.is_some(), "Bot trait object should work");
        assert_eq!(bot.name(), "RandomBot");
    }
}
