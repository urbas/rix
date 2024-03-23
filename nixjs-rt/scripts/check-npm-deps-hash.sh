#!/usr/bin/env bash

expectedHash=$(prefetch-npm-deps package-lock.json 2> /dev/null)

grep -q --fixed-string "$expectedHash" pkg.nix \
  && ( echo "✅ The 'npmDepsHash' attribute in 'pkg.nix' is up-to-date." ) \
  || ( echo "❌ The 'npmDepsHash' attribute in 'pkg.nix' is not up-to-date (expected: '$expectedHash')." && exit 1 )
