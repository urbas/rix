#!/usr/bin/env bash

RUST_VERSION=$(<.rust-version)
rustup toolchain install $RUST_VERSION
export PATH=$HOME/.rustup/toolchains/$(rustup toolchain list | grep --fixed-string $RUST_VERSION | head -n1)/bin:$PATH
