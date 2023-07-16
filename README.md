# rix

[![builder](https://github.com/urbas/rix/actions/workflows/build.yml/badge.svg)](https://github.com/urbas/rix/actions/workflows/build.yml)

A reimplementation of `nix` in Rust.

# Trying it out

Currently `rix` is not published anywhere, so you'll have to build it yourself.
Please follow instructions in [`CONTRIBUTE.md`](./CONTRIBUTE.md) on how to build
and run `rix`.

Keep in mind that `rix` is still in development and many features are not yet
implemented.

# Notable design choices

1. Nix expressions are transpiled to JavaScript and evaluated with V8. The idea
   is to leverage all the great work around the JS ecosystem (such as debuggers,
   fast JIT compilers, profilers, libraries, compiled code caching, source
   mapping, just to name a few).

2. Use plain-old files and directories to store metadata (instead of a central
   SQLite database). The idea is to have trully immutable stores, composable
   stores, avoid the central sqlite choke-point, and be more transparent (allow
   users to browse the store's metadata without having to learn about SQLite).

3. Shard directories that contain huge amounts of hash-prefixed files (i.e., use
   paths like `/nix/store/ca/fe/xxzzxjyhvbll1c7bkswwy36nlafx-foo-1.2.3`).

# Progress

## New sub-commands

- 🌗 `build-derivation`: builds a derivation in a sandbox.

  - 🌕 stage 0: creates a sandbox.
  - 🌕 stage 1: builds derivations without dependencies.
  - 🌗 stage 2: builds derivations with dependencies.
    - TODO: prevent internet access.
  - 🌑 stage 3: builds fixed derivations (with internet access).
  - 🌑 stage 4: builds X% of derivations in `nixpkgs` (assuming all dependencies
    are present).

- `transpile`: converts the given nix expression into JavaScript and prints it
  to stdout.

## Nix sub-commands

- 🌘 `eval`

  - 🌕 stage 0: evaluate basic expressions, rec attrsets, let bindings, `with`
    statement, functions
  - 🌕 stage 1: lazy evaluation
  - 🌘 stage 2:
    - 🌘 built-in functions (progress: 2 out of 111)
    - 🌑 derivations (hello world derivation)
  - 🌑 stage 3: full implementation (all derivations in nixpkgs, nice error
    messages, etc.)

- 🌘 `show-derivation`

  - 🌕 stage 1 (MVP): parse .drv files and dump JSON
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌕 `hash to-base32`

  - 🌕 stage 1 (MVP): conversions of non-SRI hashes
  - 🌕 stage 2: most common use cases
  - 🌕 stage 3: full implementation

- 🌕 `hash to-base64`

  - 🌕 stage 1 (MVP): conversions of non-SRI hashes
  - 🌕 stage 2: most common use cases
  - 🌕 stage 3: full implementation

- 🌕 `hash to-base16`

  - 🌕 stage 1 (MVP): conversions of non-SRI hashes
  - 🌕 stage 2: most common use cases
  - 🌕 stage 3: full implementation

- 🌕 `hash to-sri`

  - 🌕 stage 1 (MVP)
  - 🌕 stage 2: most common use cases
  - 🌕 stage 3: full implementation

- 🌑 `hash file`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `hash path`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `build`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `develop`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `flake`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `help`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `profile`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `repl`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `run`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `search`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `shell`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `bundle`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `copy`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `edit`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `log`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `path-info`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `registry`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `why-depends`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `daemon`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `describe-stores`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `key`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `nar`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `print-dev-env`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `realisation`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `show-config`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `store`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `doctor`

  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

- 🌑 `upgrade-nix`
  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation
