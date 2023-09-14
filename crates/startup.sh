#!/bin/sh
set -e
cargo test --profile docker
cargo build --profile docker
mkdir -p _artifacts
(cd _artifacts; ../target/docker/ffxiv_server)
