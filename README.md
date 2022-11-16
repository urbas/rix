# rix

[![builder](https://github.com/urbas/rix/actions/workflows/build.yml/badge.svg)](https://github.com/urbas/rix/actions/workflows/build.yml)

A reimplementation of `nix` in Rust.

# Progress

## New sub-commands

- 🌗 `build-derivation`: builds a derivation in a sandbox.
  - 🌕 stage 0: creates a sandbox.
  - 🌕 stage 1: builds derivations without dependencies.
  - 🌗 stage 2: builds derivations with dependencies.
    - TODO: mount directories for each derivation output.
    - TODO: deduplicate mount paths (rix now fails if multiple dependent derivations specify the same mount paths).
    - TODO: prevent internet access.
    - TODO: mount runtime dependencies of output paths too.
  - 🌑 stage 3: builds fixed derivations (with internet access).
  - 🌑 stage 4: builds X% of derivations in `nixpkgs` (assuming all dependencies are present).

## Nix sub-commands

- 🌘 `eval`

  - 🌕 stage 0 (evaluate basic integer arithmetic)
  - 🌑 stage 1 (MVP)
  - 🌑 stage 2: most common use cases
  - 🌑 stage 3: full implementation

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
