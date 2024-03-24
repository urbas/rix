#!/usr/bin/env bash

set -e

outLink=$(mktemp -d)/result
nix build --out-link $outLink

[[ $($outLink/bin/rix eval --expr '1+2') == "3" ]] \
  && ( echo "✅ The 'rix' executable exists and works." ) \
  || ( echo "❌ The 'rix' executable does not exist or is not working properly." && exit 1 )
