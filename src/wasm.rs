use wasm_bindgen::prelude::*;
use crate::api::{handle_api_request, handle_verify_request, handle_initial_state_request, handle_valid_moves_request, handle_is_winning_request};
use crate::states::Player;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Initialize WASM module - sets up panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Get the initial game state as JSON
///
/// # Returns
/// JSON string containing the initial game state
///
/// # Example (JavaScript)
/// ```javascript
/// import init, { get_initial_state } from './eclipse_wasm.js';
///
/// await init();
/// const stateJson = get_initial_state();
/// const state = JSON.parse(stateJson);
/// ```
#[wasm_bindgen]
pub fn get_initial_state() -> String {
    match handle_initial_state_request() {
        Ok(json) => json,
        Err(e) => format!(r#"{{"success": false, "error": "{}"}}"#, e),
    }
}

/// Get the best move for a given game state using the minimax bot
///
/// # Arguments
/// * `state_json` - JSON string representation of the game state
/// * `player` - The player to move next ("light" or "dark")
/// * `depth` - Search depth for minimax algorithm (1-7)
/// * `weight` - Evaluation weight multiplier
///
/// # Returns
/// JSON string containing the best move or an error message
///
/// # Example (JavaScript)
/// ```javascript
/// const response = get_best_move(stateJson, "light", 3, 1.0);
/// const result = JSON.parse(response);
/// if (result.success) {
///     console.log("Best move:", result.best_move);
/// }
/// ```
#[wasm_bindgen]
pub fn get_best_move(state_json: &str, player: &str, depth: u8, weight: f64) -> String {
    let player_enum = match player.to_lowercase().as_str() {
        "light" => Player::Light,
        "dark" => Player::Dark,
        _ => {
            return format!(
                r#"{{"success": false, "error": "Invalid player: '{}'. Must be 'light' or 'dark'"}}"#,
                player
            );
        }
    };

    match handle_api_request(state_json, player_enum, depth, weight) {
        Ok(json) => json,
        Err(e) => format!(r#"{{"success": false, "error": "{}"}}"#, e),
    }
}

/// Verify if a move is legal for a given game state
///
/// # Arguments
/// * `state_json` - JSON string representation of the game state
/// * `player` - The player making the move ("light" or "dark")
/// * `move_json` - JSON string representation of the move to verify
///
/// # Returns
/// JSON string indicating whether the move is legal and why
///
/// # Example (JavaScript)
/// ```javascript
/// const response = verify_move(stateJson, "light", moveJson);
/// const result = JSON.parse(response);
/// if (result.is_legal) {
///     console.log("Move is legal!");
/// } else {
///     console.log("Move is illegal:", result.reason);
/// }
/// ```
#[wasm_bindgen]
pub fn verify_move(state_json: &str, player: &str, move_json: &str) -> String {
    let player_enum = match player.to_lowercase().as_str() {
        "light" => Player::Light,
        "dark" => Player::Dark,
        _ => {
            return format!(
                r#"{{"success": false, "error": "Invalid player: '{}'. Must be 'light' or 'dark'"}}"#,
                player
            );
        }
    };

    match handle_verify_request(state_json, player_enum, move_json) {
        Ok(json) => json,
        Err(e) => format!(r#"{{"success": false, "error": "{}"}}"#, e),
    }
}

/// Get all valid moves for a player in a given game state
///
/// # Arguments
/// * `state_json` - JSON string representation of the game state
/// * `player` - The player whose moves to return ("light" or "dark")
///
/// # Returns
/// JSON string containing all valid moves or an error message
///
/// # Example (JavaScript)
/// ```javascript
/// const response = get_valid_moves(stateJson, "light");
/// const result = JSON.parse(response);
/// if (result.success) {
///     console.log("Valid moves:", result.valid_moves);
/// }
/// ```
#[wasm_bindgen]
pub fn get_valid_moves(state_json: &str, player: &str) -> String {
    let player_enum = match player.to_lowercase().as_str() {
        "light" => Player::Light,
        "dark" => Player::Dark,
        _ => {
            return format!(
                r#"{{"success": false, "error": "Invalid player: '{}'. Must be 'light' or 'dark'"}}"#,
                player
            );
        }
    };

    match handle_valid_moves_request(state_json, player_enum) {
        Ok(json) => json,
        Err(e) => format!(r#"{{"success": false, "error": "{}"}}"#, e),
    }
}

/// Check if the current position is a win for the player who just moved
///
/// # Arguments
/// * `state_json` - JSON string representation of the game state
///
/// # Returns
/// JSON string indicating whether the position is winning and who won
///
/// # Example (JavaScript)
/// ```javascript
/// const response = is_winning(stateJson);
/// const result = JSON.parse(response);
/// if (result.is_winning) {
///     console.log("Winner:", result.winner);
/// }
/// ```
#[wasm_bindgen]
pub fn is_winning(state_json: &str) -> String {
    match handle_is_winning_request(state_json) {
        Ok(json) => json,
        Err(e) => format!(r#"{{"success": false, "error": "{}"}}"#, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_initial_state() {
        let result = get_initial_state();
        assert!(result.contains("comet_light"));
        assert!(result.contains("comet_dark"));
    }

    #[test]
    fn test_get_best_move() {
        let state_json = get_initial_state();
        let result = get_best_move(&state_json, "light", 2, 1.0);
        assert!(result.contains("success"));
    }

    #[test]
    fn test_get_best_move_invalid_player() {
        let state_json = get_initial_state();
        let result = get_best_move(&state_json, "invalid", 2, 1.0);
        assert!(result.contains("error"));
        assert!(result.contains("Invalid player"));
    }

    #[test]
    fn test_get_valid_moves() {
        let state_json = get_initial_state();
        let result = get_valid_moves(&state_json, "light");
        assert!(result.contains("success"));
        assert!(result.contains("valid_moves"));
    }

    #[test]
    fn test_verify_move() {
        let state_json = get_initial_state();
        // Get valid moves first
        let moves_result = get_valid_moves(&state_json, "light");
        let moves_data: serde_json::Value = serde_json::from_str(&moves_result).unwrap();

        if let Some(moves) = moves_data["valid_moves"].as_array() {
            if let Some(first_move) = moves.first() {
                let move_json = serde_json::to_string(first_move).unwrap();
                let result = verify_move(&state_json, "light", &move_json);
                assert!(result.contains("is_legal"));
            }
        }
    }

    #[test]
    fn test_is_winning() {
        let state_json = get_initial_state();
        let result = is_winning(&state_json);
        assert!(result.contains("success"));
    }
}
