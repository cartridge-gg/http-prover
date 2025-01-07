#!/bin/bash

# This script compiles the bootloaders with python only usable in the docker
set -eux

mkdir -p bootloaders
CAIRO_COMPILE="cairo-lang/src/starkware/cairo/lang/scripts/cairo-compile"

python "$CAIRO_COMPILE" \
    cairo-lang/src/starkware/cairo/bootloaders/simple_bootloader/recursive/simple_bootloader.cairo \
    --output bootloaders/recursive.json \
    --proof_mode

python "$CAIRO_COMPILE" \
    cairo-lang/src/starkware/cairo/bootloaders/simple_bootloader/recursive_with_poseidon/simple_bootloader.cairo \
    --output bootloaders/recursive_with_poseidon.json \
    --proof_mode

python "$CAIRO_COMPILE" \
    cairo-lang/src/starkware/cairo/bootloaders/simple_bootloader/starknet/simple_bootloader.cairo \
    --output bootloaders/starknet.json \
    --proof_mode

python "$CAIRO_COMPILE" \
    cairo-lang/src/starkware/cairo/bootloaders/simple_bootloader/starknet_with_keccak/simple_bootloader.cairo \
    --output bootloaders/starknet_with_keccak.json \
    --proof_mode
