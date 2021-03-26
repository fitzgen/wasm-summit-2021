#!/usr/bin/env bash

set -ex

cd $(dirname $0)

### Without Wizer

# cargo build --target wasm32-wasi
# cp ../target/wasm32-wasi/debug/spelling-corrector.wasm control.wasm

cargo build --release --target wasm32-wasi
wasm-opt -O2 ../target/wasm32-wasi/release/spelling-corrector.wasm \
         -o ../target/wasm32-wasi/release/spelling-corrector.opt.wasm
cp ../target/wasm32-wasi/release/spelling-corrector.opt.wasm control.wasm

### With Wizer

# cargo build --features wizer --target wasm32-wasi
# wizer ../target/wasm32-wasi/debug/spelling-corrector.wasm \
#       -o ../target/wasm32-wasi/debug/spelling-corrector.wizer.wasm \
#       --allow-wasi \
#       --dir .
# cp ../target/wasm32-wasi/debug/spelling-corrector.wizer.wasm wizer.wasm

cargo build --release --features wizer --target wasm32-wasi
wasm-opt -O2 ../target/wasm32-wasi/release/spelling-corrector.wasm \
         -o ../target/wasm32-wasi/release/spelling-corrector.opt.wasm
wizer ../target/wasm32-wasi/release/spelling-corrector.opt.wasm \
      -o ../target/wasm32-wasi/release/spelling-corrector.opt.wizer.wasm \
      --allow-wasi \
      --dir .
wasm-opt -O2 \
         ../target/wasm32-wasi/release/spelling-corrector.opt.wizer.wasm \
         -o ../target/wasm32-wasi/release/spelling-corrector.opt.wizer.opt.wasm
cp ../target/wasm32-wasi/release/spelling-corrector.opt.wizer.opt.wasm wizer.wasm

### Test

wasmtime run --dir . control.wasm funciton
wasmtime run         wizer.wasm   funciton
