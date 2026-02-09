# Eclipse HTTP API Server

An HTTP API server for Eclipse game that provides move validation and bot move calculation endpoints.

## Quick Start

### 1. Build the API Server

```bash
cargo build --bin eclipse-api --release
```

### 2. Run the API Server

```bash
./target/release/eclipse-api
```

The server will start on `http://localhost:8080`

You should see:
```
╔══════════════════════════════════════════════════════════════╗
║              Eclipse API Server Starting                     ║
╚══════════════════════════════════════════════════════════════╝

Server running at: http://localhost:8080

Endpoints:
  GET  /health         - Health check
  GET  /initial_state  - Get initial game state
  POST /valid_moves    - Get all valid moves for a player
  POST /bot            - Get best move from minimax bot
  POST /verify         - Verify if a move is legal

Press Ctrl+C to stop
```

### 3. Use the Web Interface

```bash
open index.html
```

The web interface will automatically connect to the API server at `localhost:8080`.

## API Endpoints

### GET /health

Health check endpoint to verify the API server is running.

**Response:**
```json
{
  "status": "ok",
  "service": "eclipse-api",
  "version": "0.1.0"
}
```

### GET /initial_state

Get the initial game state for a new game.

**Response:**
```json
{
  "occupied": [
    {
      "hex": {"q": 3, "r": 0},
      "occupant": {"Comet": "Light"}
    },
    {
      "hex": {"q": -3, "r": 0},
      "occupant": {"Comet": "Dark"}
    },
    ...
  ],
  "chains": [
    {
      "id": 0,
      "owner": "Light",
      "ctype": "Short",
      "head": {"q": 4, "r": -3},
      "tail": {"q": 4, "r": -2},
      "head_last_moved": 0,
      "tail_last_moved": 0
    },
    ...
  ],
  "comet_light": {"q": 3, "r": 0},
  "comet_dark": {"q": -3, "r": 0},
  "current_turn": "Light",
  "status": "InProgress",
  "move_history": [],
  "status_messages": [],
  "move_number": 0,
  "comet_light_last_moved": 0,
  "comet_dark_last_moved": 0
}
```

### POST /valid_moves

Get all valid moves for a given player and game state.

**Request Body:**
```json
{
  "state": {
    "occupied": [...],
    "chains": [...],
    "comet_light": {"q": 3, "r": 0},
    "comet_dark": {"q": -3, "r": 0},
    "current_turn": "Light",
    "status": "InProgress",
    "move_history": [],
    "status_messages": [],
    "board_radius": 4
  },
  "player": "light"
}
```

**Parameters:**
- `state` (object, required): Complete game state
- `player` (string, required): Player to get moves for ("light" or "dark", case-insensitive)

**Success Response:**
```json
{
  "success": true,
  "valid_moves": [
    {
      "MoveComet": {"q": 2, "r": 0}
    },
    {
      "MoveSatellite": {
        "chain_id": 0,
        "old_pos": {"q": 4, "r": -3},
        "new_pos": {"q": 3, "r": -3}
      }
    },
    ...
  ],
  "move_count": 42
}
```

**Error Response:**
```json
{
  "success": false,
  "error": "Failed to parse game state JSON: ..."
}
```

### POST /bot

Get the best move for a given game state using the minimax algorithm.

**Request Body:**
```json
{
  "depth": 3,
  "weight": 1.0,
  "state": {
    "occupied": [...],
    "chains": [...],
    "comet_light": {"q": 3, "r": 0},
    "comet_dark": {"q": -3, "r": 0},
    "current_turn": "Light",
    "status": "InProgress",
    "move_history": [],
    "status_messages": [],
    "board_radius": 4
  },
  "next_move": "light"
}
```

**Parameters:**
- `depth` (integer, required): Search depth for minimax (2-4)
  - 2 = Easy
  - 3 = Medium
  - 4 = Hard
- `weight` (float, required): Evaluation weight multiplier (0.5-2.0)
- `state` (object, required): Complete game state
- `next_move` (string, required): Player to move ("light" or "dark", case-insensitive)

**Success Response:**
```json
{
  "success": true,
  "best_move": {
    "MoveSatellite": {
      "chain_id": 2,
      "old_pos": {"q": 4, "r": -1},
      "new_pos": {"q": 1, "r": 2}
    }
  },
  "score": 21.0,
  "legal_moves_count": 21
}
```

**Error Response:**
```json
{
  "success": false,
  "error": "Invalid depth: 5. Must be 2, 3, or 4"
}
```

### POST /verify

Verify if a move is legal for a given game state.

**Request Body:**
```json
{
  "state": {
    "occupied": [...],
    "chains": [...],
    "comet_light": {"q": 3, "r": 0},
    "comet_dark": {"q": -3, "r": 0},
    "current_turn": "Light",
    "status": "InProgress",
    "move_history": [],
    "status_messages": [],
    "board_radius": 4
  },
  "player": "light",
  "move": {
    "MoveComet": {"q": 2, "r": 0}
  }
}
```

**Parameters:**
- `state` (object, required): Complete game state
- `player` (string, required): Player making the move ("light" or "dark", case-insensitive)
- `move` (object, required): Move to verify

**Move Types:**

Comet Move:
```json
{
  "MoveComet": {"q": 2, "r": 0}
}
```

Satellite Move:
```json
{
  "MoveSatellite": {
    "chain_id": 2,
    "old_pos": {"q": 4, "r": -1},
    "new_pos": {"q": 1, "r": 2}
  }
}
```

**Success Response (Legal Move):**
```json
{
  "success": true,
  "is_legal": true,
  "move_verified": {
    "MoveComet": {"q": 2, "r": 0}
  }
}
```

**Success Response (Illegal Move):**
```json
{
  "success": true,
  "is_legal": false,
  "reason": "Target position is not adjacent to comet",
  "move_verified": {
    "MoveComet": {"q": 10, "r": 10}
  }
}
```

**Error Response:**
```json
{
  "success": false,
  "error": "Failed to parse game state JSON: ..."
}
```

## CORS Configuration

The API server is configured to accept requests from any origin for local development. This allows the web interface to make requests from `file://` URLs.

## Example Usage

### Using curl

**Health Check:**
```bash
curl http://localhost:8080/health
```

**Get Initial State:**
```bash
curl http://localhost:8080/initial_state
```

**Get Valid Moves:**
```bash
curl -X POST http://localhost:8080/valid_moves \
  -H "Content-Type: application/json" \
  -d '{
    "state": {...},
    "player": "light"
  }'
```

**Bot Move:**
```bash
curl -X POST http://localhost:8080/bot \
  -H "Content-Type: application/json" \
  -d '{
    "depth": 3,
    "weight": 1.0,
    "state": {...},
    "next_move": "light"
  }'
```

**Verify Move:**
```bash
curl -X POST http://localhost:8080/verify \
  -H "Content-Type: application/json" \
  -d '{
    "state": {...},
    "player": "light",
    "move": {"MoveComet": {"q": 2, "r": 0}}
  }'
```

### Using JavaScript (fetch)

```javascript
// Get initial state
const response = await fetch('http://localhost:8080/initial_state');
const gameState = await response.json();
console.log('Initial state:', gameState);
```

```javascript
// Get valid moves
const response = await fetch('http://localhost:8080/valid_moves', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
    },
    body: JSON.stringify({
        state: gameState,
        player: 'light'
    })
});

const result = await response.json();
if (result.success) {
    console.log('Valid moves:', result.valid_moves);
    console.log('Move count:', result.move_count);
}
```

```javascript
// Get bot move
const response = await fetch('http://localhost:8080/bot', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
    },
    body: JSON.stringify({
        depth: 3,
        weight: 1.0,
        state: gameState,
        next_move: 'light'
    })
});

const result = await response.json();
if (result.success) {
    console.log('Best move:', result.best_move);
}
```

```javascript
// Verify move
const response = await fetch('http://localhost:8080/verify', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
    },
    body: JSON.stringify({
        state: gameState,
        player: 'light',
        move: { MoveComet: { q: 2, r: 0 } }
    })
});

const result = await response.json();
if (result.success && result.is_legal) {
    console.log('Move is legal!');
} else if (result.success) {
    console.log('Move is illegal:', result.reason);
}
```

### Using Python (requests)

```python
import requests
import json

# Get initial state
response = requests.get('http://localhost:8080/initial_state')
game_state = response.json()
print('Initial state:', game_state)
```

```python
# Get valid moves
response = requests.post('http://localhost:8080/valid_moves', json={
    'state': game_state,
    'player': 'light'
})

result = response.json()
if result['success']:
    print('Valid moves:', result['valid_moves'])
    print('Move count:', result['move_count'])
```

```python
# Get bot move
response = requests.post('http://localhost:8080/bot', json={
    'depth': 3,
    'weight': 1.0,
    'state': game_state,
    'next_move': 'light'
})

result = response.json()
if result['success']:
    print('Best move:', result['best_move'])
```

```python
# Verify move
response = requests.post('http://localhost:8080/verify', json={
    'state': game_state,
    'player': 'light',
    'move': {'MoveComet': {'q': 2, 'r': 0}}
})

result = response.json()
if result['success'] and result['is_legal']:
    print('Move is legal!')
elif result['success']:
    print('Move is illegal:', result['reason'])
```

## Running in Production

### Port Configuration

To change the port, modify `src/bin/eclipse-api.rs`:

```rust
.bind(("127.0.0.1", 8080))? // Change 8080 to your desired port
```

### Background Service

Run as a background service using systemd (Linux):

Create `/etc/systemd/system/eclipse-api.service`:
```ini
[Unit]
Description=Eclipse API Server
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/path/to/eclipse
ExecStart=/path/to/eclipse/target/release/eclipse-api
Restart=always

[Install]
WantedBy=multi-user.target
```

Then:
```bash
sudo systemctl daemon-reload
sudo systemctl enable eclipse-api
sudo systemctl start eclipse-api
```

### Using Docker

Create a `Dockerfile`:
```dockerfile
FROM rust:1.85 as builder
WORKDIR /app
COPY . .
RUN cargo build --bin eclipse-api --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/eclipse-api /usr/local/bin/
EXPOSE 8080
CMD ["eclipse-api"]
```

Build and run:
```bash
docker build -t eclipse-api .
docker run -p 8080:8080 eclipse-api
```

## Troubleshooting

### "Connection refused" Error

- Make sure the API server is running: `./target/release/eclipse-api`
- Check if port 8080 is available: `lsof -i :8080`
- Verify firewall settings allow connections to port 8080

### CORS Errors in Browser

The API server is configured to allow all origins. If you still see CORS errors:
- Make sure you're using the latest version of the API server
- Check browser console for specific error messages
- Try using a local web server instead of opening HTML directly

### Build Errors

If you get Rust version errors:
```bash
# Update time crate to compatible version
cargo update time --precise 0.3.35

# Then rebuild
cargo build --bin eclipse-api --release
```

### High CPU Usage

The minimax bot at depth 4 can be CPU-intensive. Consider:
- Using depth 2 or 3 for faster response times
- Implementing a timeout for bot requests
- Running on a more powerful machine for production use

## Performance

### Response Times

- `/health`: < 1ms
- `/verify`: < 100ms (instant move validation)
- `/bot`:
  - Depth 2: 100ms - 1s
  - Depth 3: 1s - 5s
  - Depth 4: 5s - 30s

### Concurrent Requests

The server uses Actix-Web which handles concurrent requests efficiently. Multiple users can use the API simultaneously.

### Rate Limiting

Currently no rate limiting is implemented. For production, consider adding rate limiting middleware.

## Security Considerations

### Local Development Only

This API server is designed for local development and does not include:
- Authentication
- Rate limiting
- Input size limits beyond what's reasonable for the game

### Production Deployment

If deploying to production:
1. Add authentication (API keys, OAuth, etc.)
2. Implement rate limiting
3. Add request size limits
4. Use HTTPS (TLS)
5. Run behind a reverse proxy (nginx, Caddy)
6. Implement logging and monitoring
7. Add CORS restrictions to specific origins

## Development

### Running in Development

```bash
cargo run --bin eclipse-api
```

### Hot Reloading

Use `cargo-watch` for hot reloading during development:

```bash
cargo install cargo-watch
cargo watch -x 'run --bin eclipse-api'
```

### Testing the API

```bash
# Test health endpoint
curl http://localhost:8080/health

# Test with actual game state
cargo run --example generate_state > state.json
curl -X POST http://localhost:8080/bot \
  -H "Content-Type: application/json" \
  -d @state.json
```

## See Also

- [CLI_API.md](CLI_API.md) - Original CLI-based API documentation
- [examples/web/README.md](examples/web/README.md) - Web interface documentation
- [GAME.md](GAME.md) - Game rules and mechanics
