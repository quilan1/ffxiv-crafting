#!/bin/sh
set -e

cargo fmt
cargo check
cargo test
cargo doc --no-deps --lib -p ffxiv_items -p ffxiv_universalis -p mock_traits
