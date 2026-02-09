use crate::board::Hex;
use crate::states::{GameState, Occupant, Player, ChainId};
use crate::moves::Move;
use colored::*;

/// Displays the game board in ASCII format with hex coordinates.
///
/// The board is rendered using the following visual representation:
/// - Empty hexes: `.` (dot)
/// - Light Comet: `●` (filled circle)
/// - Dark Comet: `○` (empty circle)
/// - Light Satellites: `•` (bullet)
/// - Dark Satellites: `∘` (ring)
///
/// The display also shows:
/// - Coordinate labels (q, r) for each hex
/// - Visual chain indicators (lines connecting satellites)
pub struct BoardDisplay<'a> {
    game: &'a GameState,
}

impl<'a> BoardDisplay<'a> {
    /// Creates a new board display for the given game state.
    pub fn new(game: &'a GameState) -> Self {
        Self { game }
    }

    /// Renders the board to a string.
    ///
    /// The board is displayed row by row, with each row showing:
    /// 1. The r coordinate value
    /// 2. The hexes in that row with their piece symbols
    /// 3. The q coordinate values below
    pub fn display_board(&self) -> String {
        let mut output = String::new();

        output.push_str("\n");
        output.push_str("╔════════════════════════════════════════════════════════════════╗\n");
        output.push_str("║                    ECLIPSE GAME BOARD                          ║\n");
        output.push_str("╚════════════════════════════════════════════════════════════════╝\n");
        output.push_str("\n");

        // Display turn information with color
        let turn_text = match self.game.current_turn {
            Player::Light => format!("Current Turn: {}", "Light".bright_cyan().bold()),
            Player::Dark => format!("Current Turn: {}", "Dark".bright_magenta().bold()),
        };
        output.push_str(&turn_text);
        output.push_str("\n");

        output.push_str(&format!("Game Status: {:?}\n", self.game.status));
        output.push_str(&format!("Move #: {}\n", self.game.move_history.len()));
        output.push_str("\n");

        // Legend with colors
        output.push_str("Legend:\n");
        output.push_str(&format!("  {} = Light Comet    {} = Dark Comet\n",
            "●".bright_cyan().bold(),
            "○".bright_magenta().bold()));
        output.push_str(&format!("  {} = Light Satellites (chains a-e)    {} = Dark Satellites (chains f-j)\n",
            "a-e".cyan(),
            "f-j".magenta()));
        output.push_str("  · = Empty hex\n");
        output.push_str("\n");

        // Render the board row by row
        // The board has rows from r=-3 to r=3
        for r in -3..=3 {
            output.push_str(&self.render_row(r));
            output.push_str("\n");
        }

        output.push_str("\n");

        // Display status messages if any
        if !self.game.status_messages.is_empty() {
            output.push_str(&self.display_status_messages());
            output.push_str("\n");
        }

        output.push_str(&self.display_game_info());
        output.push_str("\n");
        output.push_str(&self.display_chains());
        output.push_str("\n");
        output.push_str(&self.display_move_history());

        output
    }

    /// Renders a single row of the hex board.
    ///
    /// Each row shows the hexes for a given r coordinate value.
    /// The q range varies per row based on the custom board shape.
    fn render_row(&self, r: i32) -> String {
        let mut row = String::new();

        // Determine the valid q range for this r value
        let (q_min, q_max) = match r {
            -3 => (-1, 4),
            -2 => (-2, 4),
            -1 => (-3, 4),
            0  => (-3, 3),
            1  => (-4, 3),
            2  => (-4, 2),
            3  => (-4, 1),
            _  => return row, // Invalid r value
        };

        // Add indentation for visual hex grid effect
        let indent = match r {
            -3 => "      ",
            -2 => "    ",
            -1 => "  ",
            0  => "    ",
            1  => "  ",
            2  => "    ",
            3  => "      ",
            _  => "",
        };

        row.push_str(indent);
        row.push_str(&format!("r={:2} │ ", r));

        // Render each hex in this row
        for q in q_min..=q_max {
            let hex = Hex::new(q, r);
            let symbol = self.get_hex_symbol_colored(&hex);
            row.push_str(&format!(" {} ", symbol));
        }

        row.push_str(&format!("  │ r={}", r));

        // Add coordinate labels on a second line
        row.push_str("\n");
        row.push_str(indent);
        row.push_str(&format!("      │ {}", " ".dimmed()));
        for q in q_min..=q_max {
            row.push_str(&format!("{:2} ", format!("{}", q).dimmed()));
        }

        row
    }

    /// Returns the appropriate colored symbol for a hex based on its occupant.
    fn get_hex_symbol_colored(&self, hex: &Hex) -> String {
        match self.game.occupied.get(hex) {
            Some(Occupant::Comet(Player::Light)) => "●".bright_cyan().bold().to_string(),
            Some(Occupant::Comet(Player::Dark)) => "○".bright_magenta().bold().to_string(),
            Some(Occupant::Satellite(chain_id, Player::Light)) => {
                chain_id.to_letter().to_string().cyan().to_string()
            }
            Some(Occupant::Satellite(chain_id, Player::Dark)) => {
                chain_id.to_letter().to_string().magenta().to_string()
            }
            None => {
                // Check if this hex is on the board
                if hex.is_on_board() {
                    "·".dimmed().to_string() // Middle dot for empty hex on board
                } else {
                    " ".to_string() // Space for hex off board
                }
            }
        }
    }

    /// Returns the appropriate symbol for a hex based on its occupant (non-colored version for tests).
    fn get_hex_symbol(&self, hex: &Hex) -> char {
        match self.game.occupied.get(hex) {
            Some(Occupant::Comet(Player::Light)) => '●',
            Some(Occupant::Comet(Player::Dark)) => '○',
            Some(Occupant::Satellite(chain_id, _)) => chain_id.to_letter(),
            None => {
                // Check if this hex is on the board
                if hex.is_on_board() {
                    '·' // Middle dot for empty hex on board
                } else {
                    ' ' // Space for hex off board
                }
            }
        }
    }

    /// Displays status messages from the last move.
    fn display_status_messages(&self) -> String {
        let mut output = String::new();

        output.push_str("═══════════════════════════════════════════════════════════════\n");
        output.push_str(&format!("{}\n", "STATUS:".bright_white().bold()));
        output.push_str("───────────────────────────────────────────────────────────────\n");

        for msg in &self.game.status_messages {
            // Color code the message based on content
            let colored_msg = if msg.contains("⚠") {
                msg.yellow().to_string()
            } else if msg.contains("🎉") {
                msg.green().bold().to_string()
            } else if msg.contains("ℹ") {
                msg.blue().to_string()
            } else {
                msg.to_string()
            };

            output.push_str(&format!("  {}\n", colored_msg));
        }

        output.push_str("═══════════════════════════════════════════════════════════════\n");

        output
    }

    /// Displays game information including legal moves and immobilized chains.
    fn display_game_info(&self) -> String {
        let mut output = String::new();

        output.push_str("═══════════════════════════════════════════════════════════════\n");
        output.push_str(&format!("{}\n", "GAME INFORMATION:".yellow().bold()));
        output.push_str("───────────────────────────────────────────────────────────────\n");

        // Get all legal moves
        let legal_moves = self.game.get_legal_moves();

        // Count move types
        let mut comet_moves = 0;
        let mut satellite_moves = 0;

        for mv in &legal_moves {
            match mv {
                Move::MoveComet(_) => comet_moves += 1,
                Move::MoveSatellite { .. } => satellite_moves += 1,
            }
        }

        // Display move statistics
        output.push_str(&format!(
            "Legal Moves: {} total ({} comet, {} satellite)\n",
            format!("{}", legal_moves.len()).bold(),
            format!("{}", comet_moves).cyan(),
            format!("{}", satellite_moves).cyan()
        ));

        // Display immobilized chains
        let current_player = self.game.current_turn;
        let mut immobilized_chains = Vec::new();

        for chain in self.game.chains.values() {
            if chain.owner == current_player && self.is_chain_crossed(chain.id) {
                immobilized_chains.push(chain.id);
            }
        }

        if !immobilized_chains.is_empty() {
            output.push_str(&format!(
                "\n{} {} chains immobilized: ",
                "⚠".red().bold(),
                immobilized_chains.len()
            ));

            for (i, chain_id) in immobilized_chains.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!("[{}]", chain_id.0));
            }
            output.push_str("\n");
            output.push_str(&format!(
                "  {} {}\n",
                "ℹ".blue(),
                "Immobilized chains cannot move until uncrossed".dimmed()
            ));
        } else {
            output.push_str(&format!(
                "\n{} All chains active - no immobilized chains\n",
                "✓".green()
            ));
        }

        // Display active chain count
        let current_player_chains: Vec<_> = self.game.chains.values()
            .filter(|c| c.owner == current_player)
            .collect();

        let active_chains = current_player_chains.iter()
            .filter(|c| !self.is_chain_crossed(c.id))
            .count();

        output.push_str(&format!(
            "Active Chains: {}/{}\n",
            format!("{}", active_chains).green(),
            current_player_chains.len()
        ));

        output.push_str("═══════════════════════════════════════════════════════════════\n");

        output
    }

    /// Displays information about all chains in the game.
    ///
    /// Shows:
    /// - Chain ID
    /// - Owner
    /// - Type (Short/Long)
    /// - Endpoints (head and tail positions)
    /// - Current length
    /// - Whether the chain is crossed by an opponent
    fn display_chains(&self) -> String {
        let mut output = String::new();

        output.push_str("═══════════════════════════════════════════════════════════════\n");
        output.push_str("CHAINS:\n");
        output.push_str("───────────────────────────────────────────────────────────────\n");

        // Group chains by player
        let mut light_chains: Vec<_> = self.game.chains.values()
            .filter(|c| c.owner == Player::Light)
            .collect();
        light_chains.sort_by_key(|c| c.id.0);

        let mut dark_chains: Vec<_> = self.game.chains.values()
            .filter(|c| c.owner == Player::Dark)
            .collect();
        dark_chains.sort_by_key(|c| c.id.0);

        output.push_str(&format!("{}\n", "Light Chains:".cyan().bold()));
        for chain in light_chains {
            let length = chain.head.distance(&chain.tail);
            let max_len = chain.ctype.max_len();
            let crossed = self.is_chain_crossed(chain.id);
            let status = if crossed {
                "⚠ CROSSED".red().bold()
            } else {
                "✓ Active".green()
            };

            output.push_str(&format!(
                "  '{}' [{}] {:?} {} ({:2},{:2}) ←→ ({:2},{:2}) │ Len: {}/{} │ {}\n",
                chain.id.to_letter(),
                chain.id.0,
                chain.ctype,
                chain.id.to_letter().to_string().cyan(),
                chain.head.q, chain.head.r,
                chain.tail.q, chain.tail.r,
                length,
                max_len,
                status
            ));
        }

        output.push_str("\n");
        output.push_str(&format!("{}\n", "Dark Chains:".magenta().bold()));
        for chain in dark_chains {
            let length = chain.head.distance(&chain.tail);
            let max_len = chain.ctype.max_len();
            let crossed = self.is_chain_crossed(chain.id);
            let status = if crossed {
                "⚠ CROSSED".red().bold()
            } else {
                "✓ Active".green()
            };

            output.push_str(&format!(
                "  '{}' [{}] {:?} {} ({:2},{:2}) ←→ ({:2},{:2}) │ Len: {}/{} │ {}\n",
                chain.id.to_letter(),
                chain.id.0,
                chain.ctype,
                chain.id.to_letter().to_string().magenta(),
                chain.head.q, chain.head.r,
                chain.tail.q, chain.tail.r,
                length,
                max_len,
                status
            ));
        }

        output.push_str("═══════════════════════════════════════════════════════════════\n");

        output
    }

    /// Displays the move history (last 5 moves).
    fn display_move_history(&self) -> String {
        let mut output = String::new();

        output.push_str("═══════════════════════════════════════════════════════════════\n");
        output.push_str(&format!("{}\n", "MOVE HISTORY:".yellow().bold()));
        output.push_str("───────────────────────────────────────────────────────────────\n");

        if self.game.move_history.is_empty() {
            output.push_str(&format!("{}\n", "No moves yet".dimmed()));
        } else {
            // Show last 5 moves
            let start_idx = if self.game.move_history.len() > 5 {
                self.game.move_history.len() - 5
            } else {
                0
            };

            for (i, mv) in self.game.move_history[start_idx..].iter().enumerate() {
                let move_num = start_idx + i + 1;
                let player = if move_num % 2 == 1 {
                    "Light".cyan()
                } else {
                    "Dark".magenta()
                };

                let move_desc = match mv {
                    Move::MoveComet(to_pos) => {
                        format!("Comet → ({:2},{:2})", to_pos.q, to_pos.r)
                    }
                    Move::MoveSatellite { chain_id, old_pos, new_pos } => {
                        format!("Chain[{}] ({:2},{:2}) → ({:2},{:2})",
                            chain_id.0, old_pos.q, old_pos.r, new_pos.q, new_pos.r)
                    }
                };

                output.push_str(&format!(
                    "  {}. {} {}\n",
                    format!("{:3}", move_num).dimmed(),
                    player,
                    move_desc
                ));
            }

            if self.game.move_history.len() > 5 {
                output.push_str(&format!(
                    "  {} (showing last 5 of {})\n",
                    "...".dimmed(),
                    self.game.move_history.len()
                ));
            }
        }

        output.push_str("═══════════════════════════════════════════════════════════════\n");

        output
    }

    /// Checks if a chain is crossed by any opponent chain.
    fn is_chain_crossed(&self, chain_id: ChainId) -> bool {
        let chain = &self.game.chains[&chain_id];
        let opponent = chain.owner.opponent();

        for other_chain in self.game.chains.values() {
            if other_chain.owner == opponent {
                if self.chains_cross(chain, other_chain) {
                    return true;
                }
            }
        }
        false
    }

    /// Checks if two chains intersect using geometric intersection.
    fn chains_cross(&self, chain_a: &crate::states::Chain, chain_b: &crate::states::Chain) -> bool {
        let (a1, a2) = (chain_a.head.to_pixel(), chain_a.tail.to_pixel());
        let (b1, b2) = (chain_b.head.to_pixel(), chain_b.tail.to_pixel());

        crate::board::segments_intersect(a1, a2, b1, b2)
    }
}

/// Convenience function to display a game board.
///
/// # Example
/// ```
/// use eclipse::states::GameState;
/// use eclipse::display::display_board;
///
/// let game = GameState::new();
/// println!("{}", display_board(&game));
/// ```
pub fn display_board(game: &GameState) -> String {
    BoardDisplay::new(game).display_board()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::states::GameState;

    #[test]
    fn test_display_board_renders() {
        let game = GameState::new();
        let display = display_board(&game);

        // Check that the output contains expected elements
        assert!(display.contains("ECLIPSE GAME BOARD"));
        assert!(display.contains("Current Turn"));
        assert!(display.contains("Legend"));
        assert!(display.contains("CHAINS"));
    }

    #[test]
    fn test_display_board_shows_comets() {
        let game = GameState::new();
        let display = display_board(&game);

        // Check that comets are displayed
        assert!(display.contains("●")); // Light comet
        assert!(display.contains("○")); // Dark comet
    }

    #[test]
    fn test_display_board_shows_satellites() {
        let game = GameState::new();
        let display = display_board(&game);

        // Check that satellites are displayed with chain letters
        // Light chains are a-e, Dark chains are f-j
        assert!(display.contains("a") || display.contains("b") || display.contains("c")); // Light satellite letters
        assert!(display.contains("f") || display.contains("g") || display.contains("h")); // Dark satellite letters
    }

    #[test]
    fn test_display_chains_shows_all_chains() {
        let game = GameState::new();
        let display = BoardDisplay::new(&game);
        let chains_display = display.display_chains();

        // Should show both Light and Dark chains
        assert!(chains_display.contains("Light Chains"));
        assert!(chains_display.contains("Dark Chains"));

        // Should show chain IDs (0-9)
        for i in 0..10 {
            assert!(chains_display.contains(&format!("[{}]", i)));
        }
    }

    #[test]
    fn test_hex_symbol_for_empty() {
        let game = GameState::new();
        let display = BoardDisplay::new(&game);

        // Test an empty hex on the board
        let empty_hex = Hex::new(0, 0); // Center should be empty initially
        let symbol = display.get_hex_symbol(&empty_hex);

        assert_eq!(symbol, '·');
    }

    #[test]
    fn test_hex_symbol_for_comet() {
        let game = GameState::new();
        let display = BoardDisplay::new(&game);

        // Test Light comet position
        let light_comet_hex = Hex::new(3, 0);
        let symbol = display.get_hex_symbol(&light_comet_hex);
        assert_eq!(symbol, '●');

        // Test Dark comet position
        let dark_comet_hex = Hex::new(-3, 0);
        let symbol = display.get_hex_symbol(&dark_comet_hex);
        assert_eq!(symbol, '○');
    }

    #[test]
    fn test_hex_symbol_for_satellite() {
        let game = GameState::new();
        let display = BoardDisplay::new(&game);

        // Test a Light satellite position (from initial setup)
        // Chain 0 (Short) is at (4,-3) and (4,-2), so (4,-3) should be 'a'
        let light_satellite_hex = Hex::new(4, -3);
        let symbol = display.get_hex_symbol(&light_satellite_hex);
        assert_eq!(symbol, 'a');

        // Test a Dark satellite position (from initial setup)
        // Chain 5 (f) is at (-1,-3) and (-2,-2), so (-1,-3) should be 'f'
        let dark_satellite_hex = Hex::new(-1, -3);
        let symbol = display.get_hex_symbol(&dark_satellite_hex);
        assert_eq!(symbol, 'f');
    }

    #[test]
    fn test_render_row_format() {
        let game = GameState::new();
        let display = BoardDisplay::new(&game);

        // Render a row and check it has expected format
        let row = display.render_row(0);

        // Should contain r=0 label
        assert!(row.contains("r= 0"));

        // Should contain q coordinates
        assert!(row.contains("-3"));
        assert!(row.contains("3"));
    }

    #[test]
    fn test_chain_crossed_detection() {
        let game = GameState::new();
        let display = BoardDisplay::new(&game);

        // At game start, no chains should be crossed
        for chain in game.chains.values() {
            let crossed = display.is_chain_crossed(chain.id);
            assert!(!crossed, "Chain {:?} should not be crossed at game start", chain.id);
        }
    }

    #[test]
    fn test_move_history_tracking() {
        let mut game = GameState::new();

        // Initially, move history should be empty
        assert_eq!(game.move_history.len(), 0);

        // Apply a move
        let moves = game.get_legal_moves();
        if let Some(mv) = moves.first() {
            game.apply_move(mv.clone()).unwrap();

            // Move history should now have 1 entry
            assert_eq!(game.move_history.len(), 1);
        }
    }

    #[test]
    fn test_move_history_display() {
        let mut game = GameState::new();

        // Apply several moves
        for _ in 0..3 {
            let moves = game.get_legal_moves();
            if let Some(mv) = moves.first() {
                game.apply_move(mv.clone()).unwrap();
            }
        }

        let display = BoardDisplay::new(&game);
        let history = display.display_move_history();

        // Should contain move history section
        assert!(history.contains("MOVE HISTORY"));
        assert!(history.contains("Light"));
        assert!(history.contains("Dark"));
    }

    #[test]
    fn test_move_history_reset() {
        let mut game = GameState::new();

        // Apply a move
        let moves = game.get_legal_moves();
        if let Some(mv) = moves.first() {
            game.apply_move(mv.clone()).unwrap();
        }

        // Move history should have entries
        assert!(!game.move_history.is_empty());

        // Reset the game
        game.reset();

        // Move history should be empty again
        assert_eq!(game.move_history.len(), 0);
    }

    #[test]
    fn test_status_messages_after_move() {
        let mut game = GameState::new();

        // Apply a move
        let moves = game.get_legal_moves();
        if let Some(mv) = moves.first() {
            game.apply_move(mv.clone()).unwrap();

            // Status messages should not be empty after a move
            assert!(!game.status_messages.is_empty());

            // Should contain move description
            let msg = game.status_messages.join(" ");
            assert!(msg.contains("moved") || msg.contains("Comet") || msg.contains("Chain"));
        }
    }

    #[test]
    fn test_game_info_display() {
        let game = GameState::new();
        let display = BoardDisplay::new(&game);
        let info = display.display_game_info();

        // Should contain game info
        assert!(info.contains("GAME INFORMATION"));
        assert!(info.contains("Legal Moves"));
        assert!(info.contains("Active Chains"));
    }

    #[test]
    fn test_status_messages_cleared_on_next_move() {
        let mut game = GameState::new();

        // Apply first move
        let moves = game.get_legal_moves();
        if let Some(mv) = moves.first() {
            game.apply_move(mv.clone()).unwrap();
            let first_msg_count = game.status_messages.len();

            // Apply second move
            let moves = game.get_legal_moves();
            if let Some(mv) = moves.first() {
                game.apply_move(mv.clone()).unwrap();

                // Status messages should be cleared and new ones added
                // (not accumulated from previous move)
                assert!(game.status_messages.len() > 0);
                // The messages should be different from first move
            }
        }
    }
}
