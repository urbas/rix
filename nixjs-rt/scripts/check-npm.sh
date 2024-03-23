#!/usr/bin/env bash

set -ex

npm ci
npm run fmt-check
npm run test
npm run build
