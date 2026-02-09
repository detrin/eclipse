#!/bin/bash
set -e

echo "Building Eclipse web interface..."

# Check if WASM files exist
if [ ! -d "src/pkg" ]; then
    echo "ERROR: WASM files not found in src/pkg/"
    echo "Please build the WASM module first:"
    echo "  cd .. && wasm-pack build --target web --no-default-features --features wasm"
    echo "  cp -r pkg web/src/"
    exit 1
fi

echo "WASM files found, proceeding with Astro build..."

# Build the Astro site
pnpm run astro build

echo "Build complete!"
