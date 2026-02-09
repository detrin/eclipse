use clap::{Parser, Subcommand};
use crate::states::Player;

/// Eclipse - A Two-Player Abstract Strategy Game
#[derive(Parser, Debug)]
#[command(name = "eclipse")]
#[command(about = "Eclipse - A Two-Player Abstract Strategy Game", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub mode: Option<Mode>,
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    /// Run the interactive game mode (default)
    Interactive,

    /// Bot mode - calculate best move for a given game state using minimax
    Bot {
        /// Search depth for minimax algorithm (2-4)
        #[arg(long, value_parser = clap::value_parser!(u8).range(2..=4))]
        depth: u8,

        /// Evaluation weight multiplier (0.5-2.0)
        #[arg(long, default_value = "1.0")]
        weight: f64,

        /// Game state as JSON string
        #[arg(long)]
        state: String,

        /// Player to move next (light or dark)
        #[arg(long, value_parser = parse_player)]
        next_move: Player,
    },

    /// Verify mode - validate if a move is legal for a given game state
    Verify {
        /// Game state as JSON string
        #[arg(long)]
        state: String,

        /// Player making the move (light or dark)
        #[arg(long, value_parser = parse_player)]
        player: Player,

        /// Move to verify as JSON string
        #[arg(long)]
        move_json: String,
    },
}

fn parse_player(s: &str) -> Result<Player, String> {
    match s.to_lowercase().as_str() {
        "light" => Ok(Player::Light),
        "dark" => Ok(Player::Dark),
        _ => Err(format!("Invalid player: '{}'. Must be 'light' or 'dark'", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_player() {
        assert!(matches!(parse_player("light"), Ok(Player::Light)));
        assert!(matches!(parse_player("Light"), Ok(Player::Light)));
        assert!(matches!(parse_player("LIGHT"), Ok(Player::Light)));
        assert!(matches!(parse_player("dark"), Ok(Player::Dark)));
        assert!(matches!(parse_player("Dark"), Ok(Player::Dark)));
        assert!(matches!(parse_player("DARK"), Ok(Player::Dark)));
        assert!(parse_player("invalid").is_err());
    }
}
