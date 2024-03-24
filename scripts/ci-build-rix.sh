#!/usr/bin/env bash

set -ex

(
  cd nixjs-rt;
  npm run build
)

cargo fmt --check
cargo clippy -- --deny "warnings"
cargo test
