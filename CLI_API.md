# Eclipse CLI and API Documentation

## Overview

The Eclipse game provides three modes of operation:

1. **Interactive Mode** - Play the game interactively in the terminal
2. **Bot Mode** - Programmatic access to the minimax bot for move calculation
3. **Verify Mode** - Validate whether a move is legal for a given game state

## Usage

### Interactive Mode (Default)

Run the game in interactive mode to play against various bot opponents:

```bash
./eclipse
# or explicitly:
./eclipse interactive
```

This will start the interactive game with a menu to choose your opponent type:
- Human vs Human
- Human vs Random Bot
- Human vs Simple Bot (Strategic)
- Human vs Minimax Bot (Easy/Medium/Hard)

### Bot Mode

Use bot mode to get the best move for a given game state using the minimax algorithm.

```bash
./eclipse bot --depth <DEPTH> --state <STATE_JSON> --next-move <PLAYER> [--weight <WEIGHT>]
```

#### Bot Mode Parameters

- `--depth <DEPTH>` (required): Search depth for minimax algorithm
  - Valid values: 2, 3, or 4
  - `2` = Easy difficulty (fastest, less accurate)
  - `3` = Medium difficulty (balanced)
  - `4` = Hard difficulty (slowest, most accurate)

- `--state <STATE_JSON>` (required): Game state as JSON string
  - Must be a valid JSON representation of the game state
  - See "JSON State Format" section below

- `--next-move <PLAYER>` (required): Player to move next
  - Valid values: `light` or `dark` (case-insensitive)

- `--weight <WEIGHT>` (optional): Evaluation weight multiplier
  - Default: 1.0
  - Valid range: 0.5 - 2.0
  - Higher values emphasize the evaluation function more strongly

#### Bot Mode Example

```bash
# Generate initial game state
cargo run --example generate_state > state.json

# Get best move for Light player at depth 3
./eclipse bot --depth 3 --state "$(cat state.json)" --next-move light

# Get best move for Dark player at depth 4 with higher weight
./eclipse bot --depth 4 --weight 1.5 --state "$(cat state.json)" --next-move dark
```

#### Bot Mode Response Format

**Success Response:**
```json
{
  "success": true,
  "best_move": {
    "MoveSatellite": {
      "chain_id": 2,
      "old_pos": {
        "q": 4,
        "r": -1
      },
      "new_pos": {
        "q": 1,
        "r": 2
      }
    }
  },
  "score": 21.0,
  "legal_moves_count": 21
}
```

or for a comet move:
```json
{
  "success": true,
  "best_move": {
    "MoveComet": {
      "q": 2,
      "r": 0
    }
  },
  "score": 15.0,
  "legal_moves_count": 15
}
```

**Error Response:**
```json
{
  "success": false,
  "error": "Failed to parse game state JSON: ..."
}
```

### Verify Mode

Use verify mode to validate whether a proposed move is legal for a given game state.

```bash
./eclipse verify --state <STATE_JSON> --player <PLAYER> --move-json <MOVE_JSON>
```

#### Verify Mode Parameters

- `--state <STATE_JSON>` (required): Game state as JSON string
  - Must be a valid JSON representation of the game state

- `--player <PLAYER>` (required): Player making the move
  - Valid values: `light` or `dark` (case-insensitive)

- `--move-json <MOVE_JSON>` (required): Move to verify as JSON string
  - Must be a valid JSON representation of a move
  - See "Move JSON Format" section below

#### Verify Mode Example

```bash
# Generate initial game state
cargo run --example generate_state > state.json

# Verify a satellite move
MOVE='{"MoveSatellite":{"chain_id":2,"old_pos":{"q":4,"r":-1},"new_pos":{"q":1,"r":2}}}'
./eclipse verify --state "$(cat state.json)" --player light --move-json "$MOVE"

# Verify a comet move
COMET_MOVE='{"MoveComet":{"q":2,"r":0}}'
./eclipse verify --state "$(cat state.json)" --player light --move-json "$COMET_MOVE"
```

#### Verify Mode Response Format

**Legal Move Response:**
```json
{
  "success": true,
  "is_legal": true,
  "move_verified": {
    "MoveSatellite": {
      "chain_id": 2,
      "old_pos": {
        "q": 4,
        "r": -1
      },
      "new_pos": {
        "q": 1,
        "r": 2
      }
    }
  }
}
```

**Illegal Move Response:**
```json
{
  "success": true,
  "is_legal": false,
  "reason": "Target position is occupied",
  "move_verified": {
    "MoveComet": {
      "q": 2,
      "r": 0
    }
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

## JSON Formats

### Game State JSON Format

The game state is represented as a JSON object with the following structure:

```json
{
  "occupied": [
    {
      "hex": { "q": 3, "r": 0 },
      "occupant": { "Comet": "Light" }
    },
    {
      "hex": { "q": 4, "r": -3 },
      "occupant": { "Satellite": [0, "Light"] }
    }
    // ... more occupied hexes
  ],
  "chains": [
    {
      "id": 0,
      "owner": "Light",
      "ctype": "Short",
      "head": { "q": 4, "r": -3 },
      "tail": { "q": 4, "r": -2 }
    }
    // ... more chains
  ],
  "comet_light": { "q": 3, "r": 0 },
  "comet_dark": { "q": -3, "r": 0 },
  "current_turn": "Light",
  "status": "InProgress",
  "move_history": [],
  "status_messages": [],
  "board_radius": 4
}
```

#### State Fields

- `occupied`: Array of occupied hex positions with their occupants
  - Each entry has `hex` (coordinates) and `occupant` (piece type and owner)

- `chains`: Array of all chain pieces
  - `id`: Unique chain identifier (0-9)
  - `owner`: Player who owns the chain ("Light" or "Dark")
  - `ctype`: Chain type ("Short" or "Long")
  - `head`, `tail`: Positions of the two ends of the chain

- `comet_light`, `comet_dark`: Positions of each player's comet

- `current_turn`: Which player's turn it is (overridden by `--next-move` in bot mode, `--player` in verify mode)

- `status`: Game status ("InProgress" or {"Won": "Player"})

- `move_history`: List of moves made so far

- `status_messages`: Status messages from the last move

- `board_radius`: Board size (usually 4)

### Move JSON Format

Moves are represented as JSON objects with one of two formats:

**Comet Move:**
```json
{
  "MoveComet": {
    "q": 2,
    "r": 0
  }
}
```

**Satellite Move:**
```json
{
  "MoveSatellite": {
    "chain_id": 2,
    "old_pos": {
      "q": 4,
      "r": -1
    },
    "new_pos": {
      "q": 1,
      "r": 2
    }
  }
}
```

## Generating Game State JSON

To generate the initial game state JSON, use the included example:

```bash
cargo run --example generate_state > initial_state.json
```

To get the state after making moves, you can:
1. Play the interactive game and serialize the state programmatically
2. Manually construct the JSON following the format above
3. Apply moves to an existing state and re-serialize it

## Integration Examples

### Using Bot Mode in Scripts

```bash
#!/bin/bash

# Initialize game state
STATE=$(cargo run --example generate_state 2>/dev/null)

# Get best move
RESPONSE=$(./eclipse bot --depth 3 --state "$STATE" --next-move light)

# Check if successful
if echo "$RESPONSE" | jq -e '.success' > /dev/null; then
    # Extract best move
    MOVE=$(echo "$RESPONSE" | jq '.best_move')
    echo "Best move: $MOVE"
else
    # Handle error
    ERROR=$(echo "$RESPONSE" | jq -r '.error')
    echo "Error: $ERROR"
fi
```

### Using Verify Mode in Scripts

```bash
#!/bin/bash

STATE=$(cat state.json)
MOVE='{"MoveComet":{"q":2,"r":0}}'

# Verify move
RESPONSE=$(./eclipse verify --state "$STATE" --player light --move-json "$MOVE")

# Check if move is legal
IS_LEGAL=$(echo "$RESPONSE" | jq -r '.is_legal')

if [ "$IS_LEGAL" = "true" ]; then
    echo "Move is legal!"
else
    REASON=$(echo "$RESPONSE" | jq -r '.reason')
    echo "Move is illegal: $REASON"
fi
```

### Using Bot Mode from Python

```python
import subprocess
import json

def get_best_move(state_json, player, depth=3, weight=1.0):
    """Get the best move using the Eclipse minimax bot."""
    cmd = [
        "./eclipse",
        "bot",
        "--depth", str(depth),
        "--weight", str(weight),
        "--state", state_json,
        "--next-move", player
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)
    response = json.loads(result.stdout)

    if response["success"]:
        return response["best_move"]
    else:
        raise Exception(response["error"])

# Example usage
with open("initial_state.json") as f:
    state = f.read()

best_move = get_best_move(state, "light", depth=3)
print(f"Best move: {best_move}")
```

### Using Verify Mode from Python

```python
import subprocess
import json

def verify_move(state_json, player, move):
    """Verify if a move is legal."""
    move_json = json.dumps(move)

    cmd = [
        "./eclipse",
        "verify",
        "--state", state_json,
        "--player", player,
        "--move-json", move_json
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)
    response = json.loads(result.stdout)

    if not response["success"]:
        raise Exception(response["error"])

    return {
        "is_legal": response["is_legal"],
        "reason": response.get("reason")
    }

# Example usage
with open("state.json") as f:
    state = f.read()

move = {
    "MoveComet": {"q": 2, "r": 0}
}

result = verify_move(state, "light", move)
if result["is_legal"]:
    print("Move is legal!")
else:
    print(f"Move is illegal: {result['reason']}")
```

### Using Bot and Verify Modes from Node.js

```javascript
const { exec } = require('child_process');
const util = require('util');
const execPromise = util.promisify(exec);

async function getBestMove(stateJson, player, depth = 3, weight = 1.0) {
  const cmd = [
    './eclipse',
    'bot',
    '--depth', depth,
    '--weight', weight,
    '--state', `'${stateJson}'`,
    '--next-move', player
  ].join(' ');

  const { stdout } = await execPromise(cmd);
  const response = JSON.parse(stdout);

  if (response.success) {
    return response.best_move;
  } else {
    throw new Error(response.error);
  }
}

async function verifyMove(stateJson, player, move) {
  const moveJson = JSON.stringify(move);

  const cmd = [
    './eclipse',
    'verify',
    '--state', `'${stateJson}'`,
    '--player', player,
    '--move-json', `'${moveJson}'`
  ].join(' ');

  const { stdout } = await execPromise(cmd);
  const response = JSON.parse(stdout);

  if (!response.success) {
    throw new Error(response.error);
  }

  return {
    isLegal: response.is_legal,
    reason: response.reason
  };
}

// Example usage
const fs = require('fs');
const state = fs.readFileSync('initial_state.json', 'utf8');

getBestMove(state, 'light', 3, 1.0)
  .then(move => console.log('Best move:', move))
  .catch(err => console.error('Error:', err));

const move = { MoveComet: { q: 2, r: 0 } };
verifyMove(state, 'light', move)
  .then(result => {
    if (result.isLegal) {
      console.log('Move is legal!');
    } else {
      console.log('Move is illegal:', result.reason);
    }
  })
  .catch(err => console.error('Error:', err));
```

## Performance Notes

### Bot Mode
- **Depth 2 (Easy)**: Usually completes in < 1 second
- **Depth 3 (Medium)**: Usually completes in 1-5 seconds
- **Depth 4 (Hard)**: May take 5-30 seconds depending on game state complexity

The minimax algorithm uses alpha-beta pruning to improve performance, but deeper searches will naturally take longer.

### Verify Mode
- **Instant**: Move verification is very fast (< 100ms) as it only checks if the move is in the list of legal moves

## Common Error Messages

### Bot Mode Errors

1. **"Failed to parse game state JSON"**
   - Ensure JSON is properly formatted
   - Check that all required fields are present
   - Verify hex coordinates and occupant types are valid

2. **"Invalid depth"**
   - Depth must be 2, 3, or 4

3. **"Invalid player"**
   - Player must be "light" or "dark" (case-insensitive)

4. **"No legal moves available"**
   - The specified player has no legal moves in the given state
   - Game may be over

### Verify Mode Errors

1. **"Failed to parse game state JSON"**
   - Same as bot mode

2. **"Failed to parse move JSON"**
   - Ensure move JSON is properly formatted
   - Check that it matches one of the two move formats (MoveComet or MoveSatellite)

3. **"Invalid player"**
   - Player must be "light" or "dark" (case-insensitive)

### Verify Mode Reasons (when move is illegal)

- **"Target position is occupied"** - The destination hex is already occupied
- **"Target position is not on the board"** - The destination hex is outside the valid board area
- **"Target position is not adjacent to comet"** - Comets can only move to adjacent hexes
- **"Move would cross an opponent's chain"** - Comets cannot cross opponent chains
- **"Chain does not exist"** - The specified chain ID is not valid
- **"Chain belongs to opponent"** - You cannot move opponent's chains
- **"Chain is immobilized by an opponent chain"** - The chain is crossed and cannot move
- **"Position is not part of chain"** - The old_pos does not match either end of the chain
- **"New chain length does not match required length"** - Chains have fixed lengths and must maintain that length
- **"Long chains cannot be aligned along a single axis"** - Long chains must span diagonally

## Exit Codes

- `0`: Success
- `1`: Error (error message written to stderr)

## See Also

- [GAME.md](GAME.md) - Complete game rules and mechanics
- [TASK.md](TASK.md) - Development tasks and progress
- [Cargo.toml](Cargo.toml) - Project dependencies and configuration
