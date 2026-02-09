# Eclipse

> A strategic hexagonal board game where players maneuver comets and satellite chains to immobilize their opponent.

Eclipse is a complete game implementation featuring a pure Rust engine that can run as a CLI application, HTTP API server, or compile to WebAssembly for browser-based play with zero network latency.

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-Ready-blue.svg)](https://webassembly.org/)

## ✨ Features

### Game Engine
- **Pure Rust Implementation** - Fast, safe, and memory-efficient game logic
- **Minimax AI** - Sophisticated bot with configurable difficulty (depth 1-7)
  - Alpha-beta pruning for performance
  - Iterative deepening
  - Move ordering optimizations
  - Transposition tables
- **Complete Rule Set** - Full game mechanics with chain crossing and immobilization

### Deployment Options
- **🖥️ CLI Application** - Play in your terminal with colored output
- **🌐 REST API Server** - Actix-web backend on port 8080
- **🚀 WebAssembly** - 100% client-side execution in the browser
- **⚡ Zero Latency** - WASM runs at near-native speed with no network calls

### Web Interface
- **Modern Astro Frontend** - Fast, responsive web UI
- **Interactive SVG Board** - Click to select pieces and see valid moves
- **Real-time Validation** - Instant move verification
- **Adjustable AI Difficulty** - Configure bot depth and evaluation weight
- **Offline Capable** - Works without internet after first load

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Node.js 18+ (for web interface)
- wasm-pack (for WebAssembly builds)

### Play in Terminal

```bash
# Clone the repository
git clone <repository-url>
cd eclipse

# Run the CLI game
cargo run
```

### Run Web Interface (Recommended)

**Note**: Pre-built WASM files are included in the repository for quick start.

```bash
# Navigate to web directory
cd web

# Install dependencies
npm install

# Start development server
npm run dev
```

Open `http://localhost:4321` in your browser.

**If you modify Rust code**, rebuild WASM:

```bash
# From project root
wasm-pack build --target web --no-default-features --features wasm
cp -r pkg web/src/

# Commit the updated WASM files
cd web
git add src/pkg/
git commit -m "Update WASM module"
```

### Run API Server

```bash
# Start the HTTP API server
cargo run --bin eclipse-api
```

Server runs at `http://localhost:8080`

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [GAME.md](GAME.md) | Complete game rules and mechanics |
| [API_SERVER.md](API_SERVER.md) | HTTP API endpoints and usage |
| [CLI_API.md](CLI_API.md) | Command-line interface guide |
| [wasm-example/README.md](wasm-example/README.md) | Standalone WebAssembly demo |
| [web/README.md](web/README.md) | Astro web interface documentation |
| [web/DEPLOYMENT.md](web/DEPLOYMENT.md) | Deployment guide (Vercel, Netlify, etc.) |
| [PROFILING_GUIDE.md](PROFILING_GUIDE.md) | Performance profiling and optimization |
| [ENGINE_TASKS.md](ENGINE_TASKS.md) | Minimax engine implementation notes |

## 📁 Project Structure

```
eclipse/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library root
│   ├── board.rs             # Hexagonal grid system
│   ├── moves.rs             # Move generation and types
│   ├── states.rs            # Game state management
│   ├── display.rs           # Terminal rendering
│   ├── input.rs             # User input parsing
│   ├── api.rs               # Core API functions (HTTP + WASM)
│   ├── wasm.rs              # WebAssembly bindings
│   ├── bot.rs               # Bot trait definition
│   ├── randombot.rs         # Random move bot
│   ├── simplebot.rs         # Simple evaluation bot
│   ├── minimaxbot.rs        # Minimax AI with alpha-beta pruning
│   ├── cli.rs               # CLI argument parsing
│   ├── serde_helpers.rs     # JSON serialization helpers
│   └── bin/
│       └── eclipse-api.rs   # HTTP API server (Actix-web)
│
├── examples/
│   ├── bot_demo.rs          # Bot demonstration
│   ├── chain_crossing_demo.rs
│   ├── show_initial_board.rs
│   ├── generate_state.rs
│   └── test_json_state.rs
│
├── web/                     # Astro web interface
│   ├── src/
│   │   ├── pages/
│   │   │   └── index.astro  # Entry point
│   │   ├── components/
│   │   │   └── EclipseGame.astro  # Main game component
│   │   ├── lib/
│   │   │   ├── wasmApi.ts   # WASM API wrapper (active)
│   │   │   ├── api.ts       # HTTP API client (legacy)
│   │   │   ├── gameLogic.ts # Client-side utilities
│   │   │   └── hexUtils.ts  # Hex grid math
│   │   ├── types/
│   │   │   └── game.ts      # TypeScript definitions
│   │   ├── styles/
│   │   │   └── global.css   # Global styles
│   │   └── pkg/             # WASM module (committed for deployment)
│   │       ├── eclipse.js   # JS bindings
│   │       ├── eclipse_bg.wasm  # WebAssembly binary
│   │       └── eclipse.d.ts # TypeScript definitions
│   ├── public/              # Static assets
│   ├── package.json
│   ├── build.sh             # Build script for deployment
│   ├── vercel.json          # Vercel configuration
│   ├── DEPLOYMENT.md        # Deployment guide
│   └── astro.config.mjs     # Astro configuration
│
├── pkg/                     # WASM build output (gitignored at root)
│                            # Copy to web/src/pkg/ for deployment
├── target/                  # Rust build artifacts (gitignored)
├── Cargo.toml               # Rust dependencies
├── Cargo.lock
└── README.md                # This file

Note: The pkg/ directory at the root is gitignored, but web/src/pkg/ is
committed to enable deployment without needing to build Rust on hosting platforms.
```

## 🛠️ Development

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with backtrace
RUST_BACKTRACE=1 cargo run

# Benchmark minimax performance
cargo run --bin benchmark_minimax --release
```

### WASM Development

When you modify Rust code:

```bash
# 1. Build WASM module
wasm-pack build --target web --no-default-features --features wasm

# 2. Copy to web project
cp -r pkg web/src/

# 3. Test locally
cd web && npm run dev

# 4. Commit the changes (for deployment)
git add src/pkg/
git commit -m "Update WASM module"
```

**Note**: WASM files in `web/src/pkg/` are committed to the repository so deployments
don't need to build Rust. This makes deployment faster and more reliable.

### Features

The project uses Cargo features for conditional compilation:

- `server` (default) - Includes Actix-web, Tokio, for HTTP API
- `wasm` - Includes wasm-bindgen for WebAssembly compilation

```bash
# Build with server features (default)
cargo build --features server

# Build with WASM features
cargo build --no-default-features --features wasm

# Build without any optional features
cargo build --no-default-features
```

### Running Examples

```bash
# Show initial board state
cargo run --example show_initial_board

# Bot demonstration
cargo run --example bot_demo

# Chain crossing mechanics
cargo run --example chain_crossing_demo
```

## 🎮 Game Rules Summary

**Objective**: Immobilize your opponent's comet by eliminating all legal moves.

**Pieces**:
- **Comet** (large circle): Main piece that must remain mobile
- **Satellites** (small circles): Connected in pairs by chains (5 chains per player)
- **Chains**: Fixed-length connections between satellite pairs

**Movement**:
- Comets move to adjacent empty hexes (cannot cross opponent chains)
- Satellites move while maintaining fixed chain length
- Chains can cross; most recently moved chain is "on top"
- Crossed (pinned) chains cannot move

See [GAME.md](GAME.md) for complete rules.

## 🔧 Architecture

### Core Engine (Rust)
- **Board** (`board.rs`): Axial coordinate system for hexagonal grid
- **State** (`states.rs`): Game state with move history and undo
- **Moves** (`moves.rs`): Move generation and legality checking
- **Minimax Bot** (`minimaxbot.rs`): AI with alpha-beta pruning

### API Layer (`api.rs`)
Shared by both HTTP server and WASM:
- `handle_api_request` - Get best move from bot
- `handle_verify_request` - Validate move legality
- `handle_valid_moves_request` - Get all legal moves
- `handle_initial_state_request` - Get starting position

### Deployment Targets

1. **CLI** (`main.rs`)
   - Terminal-based play
   - Colored output
   - Human vs Bot

2. **HTTP API** (`bin/eclipse-api.rs`)
   - Actix-web server on port 8080
   - JSON request/response
   - CORS enabled for development

3. **WebAssembly** (`wasm.rs`)
   - Browser-based execution
   - JavaScript bindings via wasm-bindgen
   - Zero network latency

## ⚡ Performance

### Minimax Bot Performance (Release Build)
- Depth 2: ~5ms (Easy)
- Depth 3: ~50ms (Medium)
- Depth 4: ~200ms (Hard)
- Depth 5: ~1s (Very Hard)
- Depth 6: ~5s (Expert)
- Depth 7: ~20s (Master)

### WASM Performance
- Initial load: ~500ms (includes compilation)
- Valid moves: 1-5ms
- Move verification: <1ms
- Bot moves: Same as native (within 5%)

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_initial_state

# Run with backtrace
RUST_BACKTRACE=1 cargo test
```

## 📦 Dependencies

### Rust
- `serde` / `serde_json` - JSON serialization
- `rand` - Random number generation
- `clap` - CLI argument parsing
- `colored` - Terminal colors
- `actix-web` - HTTP server (optional: server feature)
- `actix-cors` - CORS middleware (optional: server feature)
- `tokio` - Async runtime (optional: server feature)
- `wasm-bindgen` - WASM bindings (optional: wasm feature)
- `console_error_panic_hook` - Better WASM errors (optional: wasm feature)

### Web Interface
- `astro` - Static site framework
- `tailwindcss` - CSS framework
- TypeScript - Type safety

## 🚢 Deployment

### Static Site (Recommended)

**WASM files are pre-built and committed** to the repository for easy deployment.

#### Quick Deploy to Vercel/Netlify

1. **Set root directory**: `web`
2. **Build command**: `bash build.sh` (or `pnpm run build`)
3. **Output directory**: `dist`
4. Deploy!

The WASM module is already included, so no Rust toolchain needed on the deployment platform.

See [web/DEPLOYMENT.md](web/DEPLOYMENT.md) for detailed instructions including:
- Vercel setup
- Netlify configuration
- GitHub Pages with Actions
- Cloudflare Pages
- Docker deployment

#### Updating WASM for Deployment

Only needed if you modified Rust code:

```bash
# Build and commit WASM files
wasm-pack build --target web --no-default-features --features wasm
cp -r pkg web/src/
cd web
git add src/pkg/
git commit -m "Update WASM module"
git push
```

#### Manual Deployment

```bash
cd web
npm run build
# Deploy dist/ directory to your hosting platform
```

### HTTP API Server

```bash
# Build server
cargo build --release --features server

# Run
./target/release/eclipse-api
```

Deploy using systemd, Docker, or your preferred method.

## 🤝 Contributing

This is a personal project, but suggestions and bug reports are welcome!

## 📄 License

See LICENSE file for details.

## 🙏 Acknowledgments

Eclipse game design by [original designer credit if applicable]

---

**Play Eclipse in your browser**: [Demo URL if deployed]

**Questions?** See the documentation in the links above or open an issue.