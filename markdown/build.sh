#!/usr/bin/env bash

set -ex

cd $(dirname $0)


### Build

# cargo build --target wasm32-wasi
# cp ../target/wasm32-wasi/debug/markdown.wasm markdown.wasm

cargo build --release --target wasm32-wasi
wasm-opt -O2 ../target/wasm32-wasi/release/markdown.wasm \
         -o ../target/wasm32-wasi/release/markdown.opt.wasm
cp ../target/wasm32-wasi/release/markdown.opt.wasm markdown.wasm

### Test

echo '# Hello, World!' | wasmtime run markdown.wasm
