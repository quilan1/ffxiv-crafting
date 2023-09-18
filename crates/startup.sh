#!/bin/sh
set -e
cargo test --tests
cargo build --profile docker
mkdir -p _artifacts
(cd _artifacts; ../target/docker/ffxiv_server)
