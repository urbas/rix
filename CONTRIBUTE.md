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

Rix uses `nixrt` (a JavaScript library), which is located in the `nixrt-rt` folder
in this repository. It needs to be built before `rix` can be run.

## Editor

You must follow the "Shell" instructions above to make sure the `.direnv` folder
is populated. After that all the needed tooling will be in the `PATH`.

Your editor should pick up the Rust toolchain as specified in
`rust-toolchain.toml`.

### VSCode

Install the [direnv vscode extension](https://github.com/direnv/direnv-vscode).

# Build, Test, and Iterate

In the first shell you can continuously build `nixjs-rt` (the Nix JavaScript run-time library):

```bash
cd nixjs-rt
npm ci
npm run build-watch
```

In the second shell continuously test `nixjs-rt`:

```bash
cd nixjs-rt
npm run test-watch
```

In the third shell continuously check and test `rix`:

```bash
cargo-watch -x clippy -x test
```

# Run `rix`

First build nixjs-rt:

```bash
cd nixjs-rt
npm ci
npm run build-watch
```

Now run `rix` in debug mode:

```bash
cargo run -- --help
cargo run -- eval --expr '1 + 1'
```

## Updating dependencies

Update tools like `rustup`, `npm`, and other dependencies:

```bash
nix flake update
```

Update the version of Rust:

1. find the latest version on https://www.rust-lang.org/
2. Replace the old version of rust in
   [`rust-toolchain.toml`](./rust-toolchain.toml) with the new version.

Update Rust dependencies:

```bash
cargo update
```

Update JavaScript dependencies:

```bash
cd nixjs-rt
npm update
```

# Troubleshooting

## Getting a cargo error after an update

If you're seeing an error like this:

```
error: the 'cargo' binary, normally provided by the 'cargo' component, is not applicable to the '<rust toolchain>' toolchain
```

Then run the following to fix it:

```bash
rustup toolchain uninstall <rust toolchain>
```
