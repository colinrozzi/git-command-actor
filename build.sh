#!/bin/bash

echo "Building git-command-actor..."
cargo build --release --target wasm32-unknown-unknown

if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "WASM file: $(pwd)/target/wasm32-unknown-unknown/release/git_command_actor.wasm"
    echo "Manifest: $(pwd)/manifest.toml"
    echo ""
    echo "To start the actor:"
    echo "theater start $(pwd)/manifest.toml"
else
    echo "Build failed!"
    exit 1
fi
