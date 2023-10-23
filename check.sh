#!/bin/sh
set -e

cargo fmt
cargo check
cargo test --tests
cargo test --doc
cargo doc --workspace --no-deps
