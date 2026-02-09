use actix_web::{web, App, HttpResponse, HttpServer, Result};
use actix_cors::Cors;
use serde::Deserialize;
use eclipse::api::{handle_api_request, handle_verify_request, handle_initial_state_request, handle_valid_moves_request, handle_is_winning_request};
use eclipse::states::Player;

/// Request payload for /bot endpoint
#[derive(Debug, Deserialize)]
struct BotRequest {
    depth: u8,
    weight: f64,
    state: serde_json::Value,
    next_move: String,
}

/// Request payload for /verify endpoint
#[derive(Debug, Deserialize)]
struct VerifyRequest {
    state: serde_json::Value,
    player: String,
    #[serde(rename = "move")]
    move_data: serde_json::Value,
}

/// Request payload for /valid_moves endpoint
#[derive(Debug, Deserialize)]
struct ValidMovesRequest {
    state: serde_json::Value,
    player: String,
}

/// Request payload for /is_winning endpoint
#[derive(Debug, Deserialize)]
struct IsWinningRequest {
    state: serde_json::Value,
}

/// Health check endpoint
async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "eclipse-api",
        "version": "0.1.0"
    })))
}

/// Bot endpoint - returns best move for given state
async fn bot(req: web::Json<BotRequest>) -> Result<HttpResponse> {
    // Parse player
    let player = match req.next_move.to_lowercase().as_str() {
        "light" => Player::Light,
        "dark" => Player::Dark,
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": format!("Invalid player: '{}'. Must be 'light' or 'dark'", req.next_move)
            })));
        }
    };

    // Validate depth
    if req.depth < 1 || req.depth > 7 {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": format!("Invalid depth: {}. Must be between 1 and 7", req.depth)
        })));
    }

    // Convert state to JSON string
    let state_json = serde_json::to_string(&req.state).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid state JSON: {}", e))
    })?;

    // Call the bot handler
    match handle_api_request(&state_json, player, req.depth, req.weight) {
        Ok(response_json) => {
            let response: serde_json::Value = serde_json::from_str(&response_json)
                .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to parse response: {}", e)))?;
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Internal error: {}", e)
            })))
        }
    }
}

/// Initial state endpoint - returns the initial game state
async fn initial_state() -> Result<HttpResponse> {
    match handle_initial_state_request() {
        Ok(response_json) => {
            let response: serde_json::Value = serde_json::from_str(&response_json)
                .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to parse response: {}", e)))?;
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Internal error: {}", e)
            })))
        }
    }
}

/// Valid moves endpoint - returns all valid moves for a player given a game state
async fn valid_moves(req: web::Json<ValidMovesRequest>) -> Result<HttpResponse> {
    // Parse player
    let player = match req.player.to_lowercase().as_str() {
        "light" => Player::Light,
        "dark" => Player::Dark,
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": format!("Invalid player: '{}'. Must be 'light' or 'dark'", req.player)
            })));
        }
    };

    // Convert state to JSON string
    let state_json = serde_json::to_string(&req.state).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid state JSON: {}", e))
    })?;

    // Call the valid moves handler
    match handle_valid_moves_request(&state_json, player) {
        Ok(response_json) => {
            let response: serde_json::Value = serde_json::from_str(&response_json)
                .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to parse response: {}", e)))?;
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Internal error: {}", e)
            })))
        }
    }
}

/// Verify endpoint - validates if a move is legal
async fn verify(req: web::Json<VerifyRequest>) -> Result<HttpResponse> {
    // Parse player
    let player = match req.player.to_lowercase().as_str() {
        "light" => Player::Light,
        "dark" => Player::Dark,
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": format!("Invalid player: '{}'. Must be 'light' or 'dark'", req.player)
            })));
        }
    };

    // Convert state to JSON string
    let state_json = serde_json::to_string(&req.state).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid state JSON: {}", e))
    })?;

    // Convert move to JSON string
    let move_json = serde_json::to_string(&req.move_data).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid move JSON: {}", e))
    })?;

    // Call the verify handler
    match handle_verify_request(&state_json, player, &move_json) {
        Ok(response_json) => {
            let response: serde_json::Value = serde_json::from_str(&response_json)
                .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to parse response: {}", e)))?;
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Internal error: {}", e)
            })))
        }
    }
}

/// Is winning endpoint - checks if the current position is winning
async fn is_winning(req: web::Json<IsWinningRequest>) -> Result<HttpResponse> {
    // Convert state to JSON string
    let state_json = serde_json::to_string(&req.state).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid state JSON: {}", e))
    })?;

    match handle_is_winning_request(&state_json) {
        Ok(response_json) => {
            let response: serde_json::Value = serde_json::from_str(&response_json)
                .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to parse response: {}", e)))?;
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Internal error: {}", e)
        }))),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              Eclipse API Server Starting                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Server running at: http://localhost:8080");
    println!();
    println!("Endpoints:");
    println!("  GET  /health         - Health check");
    println!("  GET  /initial_state  - Get initial game state");
    println!("  POST /valid_moves    - Get all valid moves for a player");
    println!("  POST /bot            - Get best move from minimax bot");
    println!("  POST /verify         - Verify if a move is legal");
    println!("  POST /is_winning     - Check if the position is winning");
    println!();
    println!("Press Ctrl+C to stop");
    println!();

    HttpServer::new(|| {
        // Configure CORS to allow requests from any origin (for local development)
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .route("/health", web::get().to(health))
            .route("/initial_state", web::get().to(initial_state))
            .route("/valid_moves", web::post().to(valid_moves))
            .route("/bot", web::post().to(bot))
            .route("/verify", web::post().to(verify))
                .route("/is_winning", web::post().to(is_winning))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
