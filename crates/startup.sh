#!/bin/sh
set -e

echo "Setting up ffxiv_server"

# Setup the database
cargo run --profile docker -p ffxiv_items

# Run the actual app
cargo build --profile docker
(mkdir -p _artifacts && cd _artifacts && ../target/docker/ffxiv_server)
