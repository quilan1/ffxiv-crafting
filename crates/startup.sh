#!/bin/sh
set -e

echo "Setting up ffxiv_server"

PROFILE="docker-no-lto"

# Setup the database
cargo run --profile $PROFILE -p ffxiv_items

# Run integration tests
cargo test --profile $PROFILE --features "docker" --tests

# Run the actual app
cargo build --profile $PROFILE
(mkdir -p _artifacts && cd _artifacts && ../target/$PROFILE/ffxiv_server)
