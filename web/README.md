# Eclipse Web Interface

A sleek, minimal web interface for playing the Eclipse hexagonal strategy game, now powered by WebAssembly for 100% client-side execution.

## Features

- **🚀 100% Client-Side** - Entire game runs in your browser using WebAssembly (no backend needed!)
- **⚡ Zero Latency** - No network calls after initial page load
- **Interactive Hex Board**: Click on pieces to see valid moves displayed as white holes
- **Bot Integration**: Play against a minimax bot with configurable depth (2-7) and weight (0.5-2.0)
- **Move Verification**: All moves are verified instantly using WASM
- **Real-time Feedback**: Immediate feedback for all actions
- **Minimal Design**: Black and white theme with yellow (Light) and purple (Dark) game pieces
- **Offline Ready**: Works without an internet connection after first load

## Prerequisites

1. **Node.js**: Version 18+ recommended
2. **Optional**: Rust and wasm-pack (only if rebuilding WASM)

## Setup

1. Install dependencies:
   ```bash
   npm install
   ```

2. Start the development server:
   ```bash
   npm run dev
   ```

3. Open your browser to `http://localhost:4321`

## Architecture

The web interface now uses **WebAssembly** for all game logic. The Rust game engine is compiled to WASM and runs directly in your browser.

### Optional: HTTP API Backend

You can optionally run the HTTP API server if you prefer:

```bash
# From the project root
cd ..
cargo run --bin eclipse-api
```

Then change the import in `EclipseGame.astro` from `wasmApi` to `api`.

### Rebuilding WASM

If you modify the Rust code:

```bash
# From the project root
wasm-pack build --target web --no-default-features --features wasm

# Copy to web project
cp -r pkg web/src/
```

## How to Play

1. **Yellow (Light) goes first** - The current turn is displayed in the top-left panel
2. **Click on a piece** to select it - Valid moves will appear as white holes on the board
3. **Click on a hole** to make your move
4. **Bot Settings**:
   - Adjust **Depth** (2-4) for bot difficulty (higher = stronger, slower)
   - Adjust **Weight** (0.5-2.0) for evaluation strength
   - Click **"Get Bot Move"** to have the bot make a move for the current player

## Game Rules

- **Comets** (large circles): Can move to adjacent hexes, cannot cross opponent chains
- **Satellites** (small circles with letters): Connected in pairs by chains
- **Chains**: Can cross each other; the most recently moved chain "pins" the other
- **Immobilized Chains**: Cannot move when pinned by an opponent's chain
- **Win Condition**: Immobilize your opponent's comet (no legal moves available)

## Technologies

- **Astro** - Static site framework
- **WebAssembly** - Rust game engine compiled to WASM
- **Rust** - Core game logic and minimax AI
- **Tailwind CSS** - Utility-first CSS framework
- **TypeScript** - Type-safe JavaScript
- **SVG** - Hex board rendering

## Project Structure

```
web/
├── src/
│   ├── components/
│   │   └── EclipseGame.astro     # Main game component
│   ├── lib/
│   │   ├── wasmApi.ts            # WASM API wrapper (active)
│   │   ├── api.ts                # HTTP API client (legacy)
│   │   ├── gameLogic.ts          # Client-side game utilities
│   │   └── hexUtils.ts           # Hex grid math
│   ├── pages/
│   │   └── index.astro           # Entry point
│   ├── styles/
│   │   └── global.css            # Global styles
│   ├── types/
│   │   └── game.ts               # TypeScript type definitions
│   └── pkg/                      # WASM module (compiled Rust)
│       ├── eclipse.js            # JS bindings
│       ├── eclipse_bg.wasm       # WebAssembly binary
│       └── eclipse.d.ts          # TypeScript definitions
├── public/                       # Static assets
└── astro.config.mjs              # Astro configuration
```

## Commands

All commands are run from the web/ directory:

| Command                   | Action                                           |
| :------------------------ | :----------------------------------------------- |
| `npm install`             | Installs dependencies                            |
| `npm run dev`             | Starts local dev server at `localhost:4321`      |
| `npm run build`           | Build your production site to `./dist/`          |
| `npm run preview`         | Preview your build locally, before deploying     |
| `npm run astro ...`       | Run CLI commands like `astro add`, `astro check` |

## WASM API Functions

All game logic runs via WebAssembly:

- `initWasm()` - Initialize the WASM module (called once at startup)
- `getInitialState()` - Get initial game state
- `getValidMoves()` - Get all valid moves for a player
- `getBotMove()` - Get best move from minimax bot (depth 2-7)
- `verifyMove()` - Verify if a move is legal

### Performance

WebAssembly provides excellent performance:
- Initial WASM load: ~500ms
- Valid moves: 1-5ms
- Bot move (depth 3): 10-100ms
- Bot move (depth 5): 100-1000ms
- Move verification: < 1ms

## 🚢 Deployment

See [DEPLOYMENT.md](DEPLOYMENT.md) for detailed deployment instructions for:
- Vercel (recommended)
- Netlify
- GitHub Pages
- Cloudflare Pages
- Docker

**Important for deployment**: WASM files in `src/pkg/` are committed to the repository for easy deployment. When you rebuild the WASM module, commit the changes:

```bash
# From project root
wasm-pack build --target web --no-default-features --features wasm
cp -r pkg web/src/

# Commit the updated WASM files
cd web
git add src/pkg/
git commit -m "Update WASM module"
```

## Troubleshooting

- **WASM fails to load**: Make sure `src/pkg/` directory exists with WASM files
- **Module not found**: Rebuild WASM with `wasm-pack build --target web --no-default-features --features wasm` and copy to `web/src/`
- **TypeScript Errors**: Run `npm run astro check` to see detailed type errors
- **Slow performance**: Try reducing bot depth (2-3 for fast games, 5-7 for strong play)
- **Deployment fails**: Check [DEPLOYMENT.md](DEPLOYMENT.md) for platform-specific instructions
