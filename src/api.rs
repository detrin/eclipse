use crate::bot::Bot;
use crate::minimaxbot::{MinimaxBot, Difficulty};
use crate::moves::Move;
use crate::states::{GameState, Player};
use serde::{Deserialize, Serialize};
use std::error::Error;

/// API response containing the best move found by the minimax bot
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    /// Whether the operation was successful
    pub success: bool,

    /// The best move found (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_move: Option<Move>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Evaluation score of the best move
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,

    /// Number of legal moves available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_moves_count: Option<usize>,
}

impl ApiResponse {
    /// Creates a successful response
    pub fn success(best_move: Move, score: f64, legal_moves_count: usize) -> Self {
        ApiResponse {
            success: true,
            best_move: Some(best_move),
            error: None,
            score: Some(score),
            legal_moves_count: Some(legal_moves_count),
        }
    }

    /// Creates an error response
    pub fn error(message: String) -> Self {
        ApiResponse {
            success: false,
            best_move: None,
            error: Some(message),
            score: None,
            legal_moves_count: None,
        }
    }
}

/// Handles API mode: calculates the best move for a given game state
///
/// # Arguments
/// * `state_json` - JSON representation of the game state
/// * `next_player` - The player who should move next
/// * `depth` - Search depth for minimax algorithm (1-7)
/// * `weight` - Evaluation weight multiplier
///
/// # Returns
/// JSON response containing the best move or an error message
pub fn handle_api_request(
    state_json: &str,
    next_player: Player,
    depth: u8,
    weight: f64,
) -> Result<String, Box<dyn Error>> {
    // Parse the game state from JSON
    let mut game_state: GameState = match serde_json::from_str(state_json) {
        Ok(state) => state,
        Err(e) => {
            let response = ApiResponse::error(format!("Failed to parse game state JSON: {}", e));
            return Ok(serde_json::to_string_pretty(&response)?);
        }
    };

    // Set the current turn to the specified player
    game_state.current_turn = next_player;

    // Get legal moves count
    let legal_moves = game_state.get_legal_moves();
    let legal_moves_count = legal_moves.len();

    // Check if there are any legal moves
    if legal_moves.is_empty() {
        let response = ApiResponse::error("No legal moves available for the current player".to_string());
        return Ok(serde_json::to_string_pretty(&response)?);
    }

    // Create a custom minimax bot with the specified parameters
    let difficulty = match depth {
        1 => Difficulty::Easy,  // Allow depth 1 for very fast games (but uses depth 2)
        2 => Difficulty::Easy,
        3 => Difficulty::Medium,
        4 => Difficulty::Hard,
        5 => Difficulty::VeryHard,
        6 => Difficulty::Expert,
        7 => Difficulty::Master,
        _ => {
            let response = ApiResponse::error(format!("Invalid depth: {}. Must be between 1 and 7", depth));
            return Ok(serde_json::to_string_pretty(&response)?);
        }
    };

    let bot = MinimaxBot::new(next_player, difficulty);

    // Find the best move
    let best_move = match bot.choose_move(&game_state) {
        Some(mv) => mv,
        None => {
            let response = ApiResponse::error("Bot failed to find a move".to_string());
            return Ok(serde_json::to_string_pretty(&response)?);
        }
    };

    // For now, we'll use a simple evaluation score
    // In a more advanced version, we could return the actual minimax score
    let score = weight * (legal_moves_count as f64);

    // Create success response
    let response = ApiResponse::success(best_move, score, legal_moves_count);

    Ok(serde_json::to_string_pretty(&response)?)
}

/// Verify response containing whether a move is legal
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
    /// Whether the operation was successful
    pub success: bool,

    /// Whether the move is legal (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_legal: Option<bool>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Reason why the move is illegal (if not legal)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// The move that was verified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_verified: Option<Move>,
}

impl VerifyResponse {
    /// Creates a successful response for a legal move
    pub fn legal(mv: Move) -> Self {
        VerifyResponse {
            success: true,
            is_legal: Some(true),
            error: None,
            reason: None,
            move_verified: Some(mv),
        }
    }

    /// Creates a successful response for an illegal move
    pub fn illegal(mv: Move, reason: String) -> Self {
        VerifyResponse {
            success: true,
            is_legal: Some(false),
            error: None,
            reason: Some(reason),
            move_verified: Some(mv),
        }
    }

    /// Creates an error response
    pub fn error(message: String) -> Self {
        VerifyResponse {
            success: false,
            is_legal: None,
            error: Some(message),
            reason: None,
            move_verified: None,
        }
    }
}

/// Handles verify mode: validates if a move is legal for a given game state
///
/// # Arguments
/// * `state_json` - JSON representation of the game state
/// * `player` - The player making the move
/// * `move_json` - JSON representation of the move to verify
///
/// # Returns
/// JSON response indicating whether the move is legal
pub fn handle_verify_request(
    state_json: &str,
    player: Player,
    move_json: &str,
) -> Result<String, Box<dyn Error>> {
    // Parse the game state from JSON
    let mut game_state: GameState = match serde_json::from_str(state_json) {
        Ok(state) => state,
        Err(e) => {
            let response = VerifyResponse::error(format!("Failed to parse game state JSON: {}", e));
            return Ok(serde_json::to_string_pretty(&response)?);
        }
    };

    // Parse the move from JSON
    let move_to_verify: Move = match serde_json::from_str(move_json) {
        Ok(mv) => mv,
        Err(e) => {
            let response = VerifyResponse::error(format!("Failed to parse move JSON: {}", e));
            return Ok(serde_json::to_string_pretty(&response)?);
        }
    };

    // Set the current turn to the specified player
    game_state.current_turn = player;

    // Get all legal moves for the player
    let legal_moves = game_state.get_legal_moves();

    // Check if the move is in the list of legal moves
    let is_legal = legal_moves.iter().any(|legal_move| {
        match (&move_to_verify, legal_move) {
            (Move::MoveComet(pos1), Move::MoveComet(pos2)) => pos1 == pos2,
            (
                Move::MoveSatellite { chain_id: id1, old_pos: old1, new_pos: new1 },
                Move::MoveSatellite { chain_id: id2, old_pos: old2, new_pos: new2 },
            ) => id1 == id2 && old1 == old2 && new1 == new2,
            _ => false,
        }
    });

    // Create appropriate response
    let response = if is_legal {
        VerifyResponse::legal(move_to_verify)
    } else {
        // Try to determine why the move is illegal
        let reason = determine_illegal_reason(&game_state, &move_to_verify, player);
        VerifyResponse::illegal(move_to_verify, reason)
    };

    Ok(serde_json::to_string_pretty(&response)?)
}

/// Attempts to determine why a move is illegal
fn determine_illegal_reason(game_state: &GameState, mv: &Move, player: Player) -> String {
    match mv {
        Move::MoveComet(target_pos) => {
            let comet_pos = if player == Player::Light {
                game_state.comet_light
            } else {
                game_state.comet_dark
            };

            // Check if target is occupied
            if game_state.occupied.contains_key(target_pos) {
                return "Target position is occupied".to_string();
            }

            // Check if target is on board
            if !target_pos.is_on_board() {
                return "Target position is not on the board".to_string();
            }

            // Check if target is adjacent
            if comet_pos.distance(target_pos) != 1 {
                return "Target position is not adjacent to comet".to_string();
            }

            // Must be crossing an opponent chain
            "Move would cross an opponent's chain".to_string()
        }
        Move::MoveSatellite { chain_id, old_pos, new_pos } => {
            // Check if chain exists
            let chain = match game_state.chains.get(chain_id) {
                Some(c) => c,
                None => return format!("Chain {:?} does not exist", chain_id),
            };

            // Check if chain belongs to player
            if chain.owner != player {
                return format!("Chain {:?} belongs to opponent", chain_id);
            }

            // Check if chain is immobilized
            if game_state.is_chain_immobilized_external(*chain_id) {
                return format!("Chain {:?} is immobilized by an opponent chain", chain_id);
            }

            // Check if old_pos is part of this chain
            if chain.head != *old_pos && chain.tail != *old_pos {
                return format!("Position ({}, {}) is not part of chain {:?}", old_pos.q, old_pos.r, chain_id);
            }

            // Check if new position is occupied
            if game_state.occupied.contains_key(new_pos) {
                return "Target position is occupied".to_string();
            }

            // Check if new position is on board
            if !new_pos.is_on_board() {
                return "Target position is not on the board".to_string();
            }

            // Check chain length constraint
            let other_end = if chain.head == *old_pos { chain.tail } else { chain.head };
            let new_length = new_pos.distance(&other_end);

            if new_length != chain.ctype.max_len() {
                return format!(
                    "New chain length {} does not match required length {} for {:?} chain",
                    new_length, chain.ctype.max_len(), chain.ctype
                );
            }

            // Long chains can be either axis-aligned or diagonal - both are valid

            "Move is not legal for an unknown reason".to_string()
        }
    }
}

/// Handles initial_state request: returns the initial game state
///
/// # Returns
/// JSON response containing the initial game state
pub fn handle_initial_state_request() -> Result<String, Box<dyn Error>> {
    let game_state = GameState::new();
    Ok(serde_json::to_string_pretty(&game_state)?)
}

/// Valid moves response containing all possible moves for a player
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidMovesResponse {
    /// Whether the operation was successful
    pub success: bool,

    /// List of all valid moves (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_moves: Option<Vec<Move>>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Number of valid moves
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_count: Option<usize>,
}

impl ValidMovesResponse {
    /// Creates a successful response
    pub fn success(valid_moves: Vec<Move>) -> Self {
        let move_count = valid_moves.len();
        ValidMovesResponse {
            success: true,
            valid_moves: Some(valid_moves),
            error: None,
            move_count: Some(move_count),
        }
    }

    /// Creates an error response
    pub fn error(message: String) -> Self {
        ValidMovesResponse {
            success: false,
            valid_moves: None,
            error: Some(message),
            move_count: None,
        }
    }
}

/// Handles valid_moves request: returns all valid moves for the current player
///
/// # Arguments
/// * `state_json` - JSON representation of the game state
/// * `player` - The player whose valid moves to return
///
/// # Returns
/// JSON response containing all valid moves for the player
pub fn handle_valid_moves_request(
    state_json: &str,
    player: Player,
) -> Result<String, Box<dyn Error>> {
    // Parse the game state from JSON
    let mut game_state: GameState = match serde_json::from_str(state_json) {
        Ok(state) => state,
        Err(e) => {
            let response = ValidMovesResponse::error(format!("Failed to parse game state JSON: {}", e));
            return Ok(serde_json::to_string_pretty(&response)?);
        }
    };

    // Set the current turn to the specified player
    game_state.current_turn = player;

    // Get all legal moves for this player
    let valid_moves = game_state.get_legal_moves();

    // Create success response
    let response = ValidMovesResponse::success(valid_moves);

    Ok(serde_json::to_string_pretty(&response)?)
}

/// Winning response indicating whether the last move resulted in a win
#[derive(Debug, Serialize, Deserialize)]
pub struct WinningResponse {
    /// Whether the operation was successful
    pub success: bool,

    /// Whether the position is a win for the player who just moved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_winning: Option<bool>,

    /// The winner if the position is a win
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner: Option<Player>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl WinningResponse {
    /// Creates a successful response
    pub fn success(is_winning: bool, winner: Option<Player>) -> Self {
        WinningResponse {
            success: true,
            is_winning: Some(is_winning),
            winner,
            error: None,
        }
    }

    /// Creates an error response
    pub fn error(message: String) -> Self {
        WinningResponse {
            success: false,
            is_winning: None,
            winner: None,
            error: Some(message),
        }
    }
}

/// Handles is_winning request: checks if the current position is a win
/// for the player who just moved.
///
/// # Arguments
/// * `state_json` - JSON representation of the game state
///
/// # Returns
/// JSON response indicating whether the position is winning and who won
pub fn handle_is_winning_request(state_json: &str) -> Result<String, Box<dyn Error>> {
    let game_state: GameState = match serde_json::from_str(state_json) {
        Ok(state) => state,
        Err(e) => {
            let response = WinningResponse::error(format!("Failed to parse game state JSON: {}", e));
            return Ok(serde_json::to_string_pretty(&response)?);
        }
    };

    let has_comet_moves = game_state.has_legal_comet_moves(game_state.current_turn);

    if has_comet_moves {
        let response = WinningResponse::success(false, None);
        Ok(serde_json::to_string_pretty(&response)?)
    } else {
        let winner = game_state.current_turn.opponent();
        let response = WinningResponse::success(true, Some(winner));
        Ok(serde_json::to_string_pretty(&response)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::states::GameState;

    #[test]
    fn test_api_response_success() {
        let game = GameState::new();
        let moves = game.get_legal_moves();

        if let Some(first_move) = moves.first() {
            let response = ApiResponse::success(first_move.clone(), 5.0, moves.len());

            assert!(response.success);
            assert!(response.best_move.is_some());
            assert!(response.error.is_none());
            assert_eq!(response.score, Some(5.0));
            assert_eq!(response.legal_moves_count, Some(moves.len()));
        }
    }

    #[test]
    fn test_api_response_error() {
        let response = ApiResponse::error("Test error".to_string());

        assert!(!response.success);
        assert!(response.best_move.is_none());
        assert_eq!(response.error, Some("Test error".to_string()));
        assert!(response.score.is_none());
    }

    #[test]
    fn test_handle_api_request_with_valid_state() {
        let game = GameState::new();
        let state_json = serde_json::to_string(&game).unwrap();

        let result = handle_api_request(&state_json, Player::Light, 2, 1.0);

        assert!(result.is_ok());

        let response_json = result.unwrap();
        let response: ApiResponse = serde_json::from_str(&response_json).unwrap();

        assert!(response.success);
        assert!(response.best_move.is_some());
    }

    #[test]
    fn test_handle_api_request_with_invalid_json() {
        let invalid_json = "{invalid json}";

        let result = handle_api_request(invalid_json, Player::Light, 2, 1.0);

        assert!(result.is_ok());

        let response_json = result.unwrap();
        let response: ApiResponse = serde_json::from_str(&response_json).unwrap();

        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[test]
    fn test_handle_api_request_with_invalid_depth() {
        let game = GameState::new();
        let state_json = serde_json::to_string(&game).unwrap();

        let result = handle_api_request(&state_json, Player::Light, 10, 1.0);

        assert!(result.is_ok());

        let response_json = result.unwrap();
        let response: ApiResponse = serde_json::from_str(&response_json).unwrap();

        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[test]
    fn test_verify_response_legal() {
        let game = GameState::new();
        let moves = game.get_legal_moves();

        if let Some(first_move) = moves.first() {
            let response = VerifyResponse::legal(first_move.clone());

            assert!(response.success);
            assert_eq!(response.is_legal, Some(true));
            assert!(response.error.is_none());
            assert!(response.reason.is_none());
            assert!(response.move_verified.is_some());
        }
    }

    #[test]
    fn test_verify_response_illegal() {
        let game = GameState::new();
        let moves = game.get_legal_moves();

        if let Some(first_move) = moves.first() {
            let response = VerifyResponse::illegal(first_move.clone(), "Test reason".to_string());

            assert!(response.success);
            assert_eq!(response.is_legal, Some(false));
            assert!(response.error.is_none());
            assert_eq!(response.reason, Some("Test reason".to_string()));
            assert!(response.move_verified.is_some());
        }
    }

    #[test]
    fn test_handle_verify_request_with_legal_move() {
        let game = GameState::new();
        let state_json = serde_json::to_string(&game).unwrap();

        // Get a legal move
        let legal_moves = game.get_legal_moves();
        if let Some(legal_move) = legal_moves.first() {
            let move_json = serde_json::to_string(&legal_move).unwrap();

            let result = handle_verify_request(&state_json, Player::Light, &move_json);

            assert!(result.is_ok());

            let response_json = result.unwrap();
            let response: VerifyResponse = serde_json::from_str(&response_json).unwrap();

            assert!(response.success);
            assert_eq!(response.is_legal, Some(true));
            assert!(response.reason.is_none());
        }
    }

    #[test]
    fn test_handle_verify_request_with_illegal_move() {
        let game = GameState::new();
        let state_json = serde_json::to_string(&game).unwrap();

        // Create an illegal move (comet to a far position)
        let illegal_move = Move::MoveComet(crate::board::Hex::new(10, 10));
        let move_json = serde_json::to_string(&illegal_move).unwrap();

        let result = handle_verify_request(&state_json, Player::Light, &move_json);

        assert!(result.is_ok());

        let response_json = result.unwrap();
        let response: VerifyResponse = serde_json::from_str(&response_json).unwrap();

        assert!(response.success);
        assert_eq!(response.is_legal, Some(false));
        assert!(response.reason.is_some());
    }

    #[test]
    fn test_handle_verify_request_with_invalid_state_json() {
        let invalid_json = "{invalid json}";
        let move_json = r#"{"MoveComet":{"q":0,"r":0}}"#;

        let result = handle_verify_request(invalid_json, Player::Light, move_json);

        assert!(result.is_ok());

        let response_json = result.unwrap();
        let response: VerifyResponse = serde_json::from_str(&response_json).unwrap();

        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[test]
    fn test_handle_verify_request_with_invalid_move_json() {
        let game = GameState::new();
        let state_json = serde_json::to_string(&game).unwrap();
        let invalid_move_json = "{invalid json}";

        let result = handle_verify_request(&state_json, Player::Light, invalid_move_json);

        assert!(result.is_ok());

        let response_json = result.unwrap();
        let response: VerifyResponse = serde_json::from_str(&response_json).unwrap();

        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[test]
    fn test_handle_is_winning_request_initial_state() {
        let game = GameState::new();
        let state_json = serde_json::to_string(&game).unwrap();

        let result = handle_is_winning_request(&state_json);

        assert!(result.is_ok());
        let response_json = result.unwrap();
        let response: WinningResponse = serde_json::from_str(&response_json).unwrap();

        assert!(response.success);
        assert_eq!(response.is_winning, Some(false));
        assert!(response.winner.is_none());
    }
}
