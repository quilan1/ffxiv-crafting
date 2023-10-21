#!/bin/sh
set -e

cargo fmt
cargo check
cargo test --tests
cargo doc --workspace --no-deps
