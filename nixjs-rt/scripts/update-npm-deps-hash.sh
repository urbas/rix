#!/usr/bin/env bash

newHash=$(prefetch-npm-deps package-lock.json 2> /dev/null)

sed -i "s,npmDepsHash = \".*\";,npmDepsHash = \"$newHash\";," pkg.nix
