# rix

[![builder](https://github.com/urbas/rix/actions/workflows/build.yml/badge.svg)](https://github.com/urbas/rix/actions/workflows/build.yml)

Nix language interpreter.

# Trying it out

Currently `rix` is not published anywhere, so you'll have to build it yourself.
Please follow instructions in [`CONTRIBUTE.md`](./CONTRIBUTE.md) on how to build
and run `rix`.

Keep in mind that `rix` is still in development and many features are not yet
implemented.

# Notable design choices

Rix transpiles Nix expressions to JavaScript and evaluates them with V8. The idea
is to leverage all the great work in the JS ecosystem (such as debuggers,
fast JIT compilers, profilers, libraries, compiled code caching, and source
mapping just to name a few).

# Progress

- 🌕 stage 0: evaluate basic expressions, rec attrsets, let bindings, `with`
  statement, functions

- 🌕 stage 1: lazy evaluation

- 🌘 stage 2:

  - 🌘 built-in functions (progress: 3 out of 111)
  - 🌑 derivations (hello world derivation)

- 🌑 stage 3: full implementation (all derivations in nixpkgs, nice error
  messages, etc.)
