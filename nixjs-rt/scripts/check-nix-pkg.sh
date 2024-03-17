#!/usr/bin/env bash

set -e

outLink=$(mktemp -d)/result
nix build --out-link $outLink ..#nixjs-rt

function expectFile() {
  local theFile=$1
  [ -f "$theFile" ] \
    && ( echo "✅ The file '$theFile' exists." ) \
    || ( echo "❌ Could not find the file '$theFile'." && exit 1 )
}

expectFile $outLink/lib/node_modules/nixjs-rt/dist/lib.mjs
expectFile $outLink/lib/node_modules/nixjs-rt/dist/lib.mjs.map
