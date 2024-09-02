#!/usr/bin/env bash

set -eux

cargo test --no-fail-fast --workspace --verbose
