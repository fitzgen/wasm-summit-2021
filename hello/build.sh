#!/usr/bin/env bash

set -ex

cd $(dirname $0)


### Build

# cargo build --target wasm32-wasi
# cp ../target/wasm32-wasi/debug/hello.wasm hello.wasm

cargo build --release --target wasm32-wasi
wasm-opt -O2 ../target/wasm32-wasi/release/hello.wasm \
         -o ../target/wasm32-wasi/release/hello.opt.wasm
cp ../target/wasm32-wasi/release/hello.opt.wasm hello.wasm

### Test

wasmtime run hello.wasm
