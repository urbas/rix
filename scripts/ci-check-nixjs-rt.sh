#!/usr/bin/env bash

cd nixjs-rt

set -ex

npm run fmt-check
npm run test
