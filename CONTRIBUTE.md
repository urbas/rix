# First-time dev env set up

## Shell

1. Install the nix package manager:
   https://nixos.org/manual/nix/stable/installation/installation.html

2. Install direnv: https://direnv.net/

Once you enter this directory in your shell, the rust tooling should be
automatically set up. You can verify this with:

```bash
# This should match the version specified in `rust-toolchain.toml`
cargo --version
```

## Editor

You must follow the "Shell" instructions above to make sure the `.direnv`
folder is populated. After that use your preferred method to put `rustup`
into your editor's `PATH` or use https://rustup.rs/.

Your editor should pick up the Rust toolchain as specified in
`rust-toolchain.toml`.

# Building & Testing

The typical Rust way:

```
cargo build
cargo test
```
