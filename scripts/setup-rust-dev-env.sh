#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

RUST_VERSION=$(<.rust-version)
rustup toolchain install $RUST_VERSION

# It would be nicer if vscode could load the version of rust from the `.rust-version` file.
sed -i "s/\"RUSTUP_TOOLCHAIN\":.*,$/\"RUSTUP_TOOLCHAIN\": \"${RUST_VERSION}\",/" $SCRIPT_DIR/../.vscode/settings.json

export PATH=$HOME/.rustup/toolchains/$(rustup toolchain list | grep --fixed-string $RUST_VERSION | head -n1)/bin:$PATH
