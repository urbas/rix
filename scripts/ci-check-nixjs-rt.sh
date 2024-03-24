#!/usr/bin/env bash

cd nixjs-rt

set -ex

npm ci
npm run fmt-check
npm run test
