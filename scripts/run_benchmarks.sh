#!/bin/bash

# This script is an example for running Thales's benchmarks.
# It requires Thales to be compiled with --features=runtime-benchmarks

export WASMTIME_BACKTRACE_DETAILS=1

./../../target/release/thales benchmark \
    --chain dev \
    --execution=wasm \
    --wasm-execution=compiled \
    --pallet "parachain_staking" \
    --extrinsic "*" \
    --steps 32 \
    --repeat 64 \
    --raw \
    --template=./frame-weight-template.hbs \
    --output /tmp/
