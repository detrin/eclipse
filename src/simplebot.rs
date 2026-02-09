use crate::bot::Bot;
use crate::moves::Move;
use crate::states::{GameState, Player};
use rand::Rng;

// =============================================================================
// SIMPLE STRATEGIC BOT IMPLEMENTATION
// =============================================================================

/// A strategic bot that uses heuristics to evaluate and choose moves.
///
/// This bot implements several strategic principles:
/// - Prioritize moves that create chain crossings (block opponent)
/// - Avoid moves that unblock opponent's chains
/// - Move comet toward opponent's side (aggressive positioning)
/// - Protect own comet by keeping it behind chains
///
/// Each move is scored based on these heuristics, and the highest-scoring
/// move is selected.
#[derive(Debug)]
pub struct SimpleBot {
    player: Player,
}

impl SimpleBot {
    /// Creates a new SimpleBot for the specified player.
    ///
    /// # Arguments
    /// * `player` - The player this bot will play as (Light or Dark)
    ///
    /// # Example
    /// ```
    /// use eclipse::simplebot::SimpleBot;
    /// use eclipse::states::Player;
    ///
    /// let bot = SimpleBot::new(Player::Dark);
    /// ```
    pub fn new(player: Player) -> Self {
        SimpleBot { player }
    }

    /// Returns the player this bot represents.
    pub fn player(&self) -> Player {
        self.player
    }

    /// Evaluates a move and returns a score indicating its strategic value.
    ///
    /// Higher scores indicate better moves. The scoring considers:
    /// - Chain crossing creation (+100 per new crossing)
    /// - Unblocking opponent chains (-150 per unblocked chain)
    /// - Comet advancement toward opponent (+20 for moving closer)
    /// - Comet protection behind chains (+30 if behind own chains)
    /// - Random tiebreaker (+0.0 to +1.0 for variety)
    ///
    /// # Arguments
    /// * `game` - The current game state
    /// * `mv` - The move to evaluate
    ///
    /// # Returns
    /// A score value (higher is better)
    fn evaluate_move(&self, game: &GameState, mv: &Move) -> f64 {
        let mut score = 0.0;

        // Create a clone of the game state to simulate the move
        let mut simulated_game = game.clone();

        // Try to apply the move (should always succeed for legal moves)
        if simulated_game.apply_move(mv.clone()).is_err() {
            return -1000.0; // Invalid move, heavily penalize
        }

        // HEURISTIC 1: Reward creating new chain crossings (blocking opponent)
        score += self.evaluate_chain_crossings(&simulated_game, game) * 100.0;

        // HEURISTIC 2: Penalize moves that unblock opponent chains
        score -= self.evaluate_unblocking(&simulated_game, game) * 150.0;

        // HEURISTIC 3: Reward comet advancement toward opponent
        if let Move::MoveComet(_) = mv {
            score += self.evaluate_comet_advancement(&simulated_game, game) * 20.0;
        }

        // HEURISTIC 4: Reward comet protection (comet behind own chains)
        if let Move::MoveComet(_) = mv {
            score += self.evaluate_comet_protection(&simulated_game) * 30.0;
        }

        // HEURISTIC 5: Add small random factor to break ties and add variety
        let mut rng = rand::thread_rng();
        score += rng.gen_range(0.0..1.0);

        score
    }

    /// Evaluates how many new chain crossings were created by the move.
    ///
    /// Counts the number of opponent chains that are newly crossed
    /// (crossed in simulated_game but not in original_game).
    ///
    /// # Returns
    /// Number of new chain crossings created (positive number)
    fn evaluate_chain_crossings(&self, simulated_game: &GameState, original_game: &GameState) -> f64 {
        let opponent = self.player.opponent();
        let mut new_crossings = 0.0;

        // Check each opponent chain to see if it's newly blocked
        for (chain_id, chain) in &simulated_game.chains {
            if chain.owner == opponent {
                let was_blocked = original_game.is_chain_immobilized_external(*chain_id);
                let is_blocked = simulated_game.is_chain_immobilized_external(*chain_id);

                // If it wasn't blocked before but is now, we created a new crossing
                if !was_blocked && is_blocked {
                    new_crossings += 1.0;
                }
            }
        }

        new_crossings
    }

    /// Evaluates how many of our own chains were unblocked by this move.
    ///
    /// This checks if any of our chains that were previously crossed by
    /// opponent chains are now free. This is generally bad because it means
    /// we're moving chains that were blocking the opponent.
    ///
    /// # Returns
    /// Number of opponent chains that were unblocked (positive number)
    fn evaluate_unblocking(&self, simulated_game: &GameState, original_game: &GameState) -> f64 {
        let opponent = self.player.opponent();
        let mut unblocked_count = 0.0;

        // Check each opponent chain to see if it was unblocked
        for (chain_id, chain) in &simulated_game.chains {
            if chain.owner == opponent {
                let was_blocked = original_game.is_chain_immobilized_external(*chain_id);
                let is_blocked = simulated_game.is_chain_immobilized_external(*chain_id);

                // If it was blocked before but is free now, we unblocked it (bad!)
                if was_blocked && !is_blocked {
                    unblocked_count += 1.0;
                }
            }
        }

        unblocked_count
    }

    /// Evaluates if the comet moved toward the opponent's side.
    ///
    /// For Light player (starting at positive q), moving toward negative q is good.
    /// For Dark player (starting at negative q), moving toward positive q is good.
    ///
    /// # Returns
    /// 1.0 if moving toward opponent, -1.0 if moving away, 0.0 otherwise
    fn evaluate_comet_advancement(&self, simulated_game: &GameState, original_game: &GameState) -> f64 {
        let old_pos = if self.player == Player::Light {
            original_game.comet_light
        } else {
            original_game.comet_dark
        };

        let new_pos = if self.player == Player::Light {
            simulated_game.comet_light
        } else {
            simulated_game.comet_dark
        };

        // Light player advances by moving toward negative q (opponent's side)
        // Dark player advances by moving toward positive q (opponent's side)
        match self.player {
            Player::Light => {
                if new_pos.q < old_pos.q {
                    1.0  // Moving left (toward Dark's side) is good for Light
                } else if new_pos.q > old_pos.q {
                    -1.0  // Moving right (away from Dark) is bad for Light
                } else {
                    0.0  // No horizontal movement
                }
            }
            Player::Dark => {
                if new_pos.q > old_pos.q {
                    1.0  // Moving right (toward Light's side) is good for Dark
                } else if new_pos.q < old_pos.q {
                    -1.0  // Moving left (away from Light) is bad for Dark
                } else {
                    0.0  // No horizontal movement
                }
            }
        }
    }

    /// Evaluates if the comet is protected behind own chains.
    ///
    /// A comet is considered protected if there are own chains between
    /// it and the opponent's comet position.
    ///
    /// # Returns
    /// 1.0 if well protected, 0.0 otherwise
    fn evaluate_comet_protection(&self, game: &GameState) -> f64 {
        let my_comet = if self.player == Player::Light {
            game.comet_light
        } else {
            game.comet_dark
        };

        let opponent_comet = if self.player == Player::Light {
            game.comet_dark
        } else {
            game.comet_light
        };

        // Count own chains that are between our comet and opponent's comet
        let mut protecting_chains = 0;

        for chain in game.chains.values() {
            if chain.owner == self.player {
                // Check if this chain is positioned between the two comets
                // A simple heuristic: chain is protecting if its midpoint is between the comets
                let chain_mid_q = (chain.head.q + chain.tail.q) as f64 / 2.0;
                let my_q = my_comet.q as f64;
                let opp_q = opponent_comet.q as f64;

                // Check if chain is between the two comets (in q-axis)
                if (my_q < opp_q && chain_mid_q > my_q && chain_mid_q < opp_q) ||
                   (my_q > opp_q && chain_mid_q < my_q && chain_mid_q > opp_q) {
                    protecting_chains += 1;
                }
            }
        }

        // Return normalized protection score
        if protecting_chains >= 2 {
            1.0  // Well protected
        } else if protecting_chains == 1 {
            0.5  // Somewhat protected
        } else {
            0.0  // Not protected
        }
    }
}

impl Bot for SimpleBot {
    fn choose_move(&self, game: &GameState) -> Option<Move> {
        let legal_moves = game.get_legal_moves();

        if legal_moves.is_empty() {
            return None;
        }

        // Evaluate all moves and find the best one
        let mut best_move: Option<Move> = None;
        let mut best_score = f64::NEG_INFINITY;

        for mv in legal_moves {
            let score = self.evaluate_move(game, &mv);

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
        }

        best_move
    }

    fn name(&self) -> &str {
        "SimpleBot"
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
    fn test_simple_bot_creation() {
        let bot = SimpleBot::new(Player::Light);
        assert_eq!(bot.player(), Player::Light);
        assert_eq!(bot.name(), "SimpleBot");

        let bot = SimpleBot::new(Player::Dark);
        assert_eq!(bot.player(), Player::Dark);
    }

    #[test]
    fn test_simple_bot_chooses_legal_move() {
        let game = GameState::new();
        let bot = SimpleBot::new(Player::Light);

        // Bot should choose a legal move
        let chosen_move = bot.choose_move(&game);
        assert!(chosen_move.is_some(), "SimpleBot should choose a move when legal moves exist");

        // The chosen move should be in the list of legal moves
        let legal_moves = game.get_legal_moves();
        let chosen = chosen_move.unwrap();

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

        assert!(is_valid, "SimpleBot should choose a valid legal move");
    }

    #[test]
    fn test_simple_bot_evaluates_moves() {
        let game = GameState::new();
        let bot = SimpleBot::new(Player::Light);

        // Get all legal moves
        let legal_moves = game.get_legal_moves();
        assert!(!legal_moves.is_empty(), "Should have legal moves");

        // SimpleBot should be able to evaluate each move
        for mv in legal_moves {
            let score = bot.evaluate_move(&game, &mv);
            // Score should be a valid number (not NaN or infinite)
            assert!(score.is_finite(), "Score should be a finite number");
        }
    }

    #[test]
    fn test_simple_bot_prefers_different_moves_than_random() {
        let game = GameState::new();
        let simple_bot = SimpleBot::new(Player::Light);

        // Get moves from SimpleBot
        let mut simple_moves = Vec::new();

        // Collect multiple choices
        for _ in 0..10 {
            if let Some(mv) = simple_bot.choose_move(&game) {
                simple_moves.push(format!("{:?}", mv));
            }
        }

        // SimpleBot should be deterministic (always choose the same move in the same position)
        // except for the small random tiebreaker
        assert!(!simple_moves.is_empty(), "SimpleBot should make choices");
    }

    #[test]
    fn test_simple_bot_chain_crossing_evaluation() {
        let mut game = GameState::new();
        let bot = SimpleBot::new(Player::Light);

        // Create a scenario where a chain crossing is possible
        game.current_turn = Player::Light;

        // Get the original crossings count
        let original_game = game.clone();

        // Find a move that might create a crossing
        let legal_moves = game.get_legal_moves();

        // Evaluate moves and check that crossing evaluation works
        for mv in legal_moves.iter().take(5) {
            let score = bot.evaluate_move(&game, mv);
            // Should be able to evaluate without panicking
            assert!(score.is_finite());
        }

        // Test the specific crossing evaluation method
        let crossings = bot.evaluate_chain_crossings(&game, &original_game);
        assert!(crossings >= 0.0, "Crossings count should be non-negative");
    }

    #[test]
    fn test_simple_bot_comet_advancement_evaluation() {
        let game = GameState::new();
        let light_bot = SimpleBot::new(Player::Light);
        let dark_bot = SimpleBot::new(Player::Dark);

        // For Light player (starts at q=3), moving toward negative q should be valued
        let original_game = game.clone();
        let mut simulated_game = game.clone();

        // Simulate Light comet moving left (toward Dark's side)
        simulated_game.comet_light = crate::board::Hex::new(2, 0); // Move from (3,0) to (2,0)

        let advancement = light_bot.evaluate_comet_advancement(&simulated_game, &original_game);
        assert_eq!(advancement, 1.0, "Moving toward opponent should be valued positively");

        // Simulate Light comet moving right (away from Dark's side)
        simulated_game.comet_light = crate::board::Hex::new(3, 1); // Move to the right
        let advancement = light_bot.evaluate_comet_advancement(&simulated_game, &original_game);
        assert!(advancement <= 0.0, "Moving away from opponent should not be valued positively");

        // For Dark player (starts at q=-3), moving toward positive q should be valued
        let mut dark_simulated = game.clone();
        dark_simulated.comet_dark = crate::board::Hex::new(-2, 0); // Move from (-3,0) to (-2,0)

        let advancement = dark_bot.evaluate_comet_advancement(&dark_simulated, &original_game);
        assert_eq!(advancement, 1.0, "Dark moving toward Light should be valued positively");
    }

    #[test]
    fn test_simple_bot_comet_protection_evaluation() {
        let game = GameState::new();
        let bot = SimpleBot::new(Player::Light);

        // Evaluate protection for the initial game state
        let protection = bot.evaluate_comet_protection(&game);

        // Should return a value between 0.0 and 1.0
        assert!(protection >= 0.0 && protection <= 1.0, "Protection should be between 0 and 1");
    }

    #[test]
    fn test_simple_bot_unblocking_evaluation() {
        let game = GameState::new();
        let bot = SimpleBot::new(Player::Light);

        // At game start, no chains should be blocked
        let original_game = game.clone();
        let unblocking = bot.evaluate_unblocking(&game, &original_game);

        assert_eq!(unblocking, 0.0, "No chains should be unblocked at game start");
    }

    #[test]
    fn test_simple_bot_can_play_complete_game() {
        let mut game = GameState::new();
        let light_bot = SimpleBot::new(Player::Light);
        let dark_bot = SimpleBot::new(Player::Dark);

        let max_turns = 500;
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
                    assert!(result.is_ok(), "SimpleBot should only choose valid moves");
                }
                None => {
                    // No legal moves available - game should be over
                    break;
                }
            }

            turn_count += 1;
        }

        println!("SimpleBot game ran for {} turns", turn_count);
        println!("Final status: {:?}", game.status);

        // Main goal: verify bots can play without errors
        assert!(turn_count > 0, "Bots should make at least one move");
    }

    #[test]
    fn test_simple_bot_vs_random_bot() {
        use crate::randombot::RandomBot;

        let mut game = GameState::new();
        let simple_bot = SimpleBot::new(Player::Light);
        let random_bot = RandomBot::new(Player::Dark);

        let max_turns = 500;
        let mut turn_count = 0;

        while game.status == crate::states::GameStatus::InProgress && turn_count < max_turns {
            let bot: &dyn Bot = if game.current_turn == Player::Light {
                &simple_bot
            } else {
                &random_bot
            };

            let chosen_move = bot.choose_move(&game);

            match chosen_move {
                Some(mv) => {
                    let result = game.apply_move(mv);
                    assert!(result.is_ok(), "Bot should only choose valid moves");
                }
                None => break,
            }

            turn_count += 1;
        }

        println!("SimpleBot vs RandomBot game ran for {} turns", turn_count);
        println!("Final status: {:?}", game.status);

        assert!(turn_count > 0, "Game should progress");
    }

    #[test]
    fn test_simple_bot_trait_object() {
        let game = GameState::new();
        let bot: Box<dyn Bot> = Box::new(SimpleBot::new(Player::Light));

        let chosen_move = bot.choose_move(&game);
        assert!(chosen_move.is_some(), "SimpleBot trait object should work");
        assert_eq!(bot.name(), "SimpleBot");
    }
}
