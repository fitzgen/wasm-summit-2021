# My Wasm Summit 2021 Talk

This repo contains demos and benchmarks used for my Wasm Summit 2021 talk!

## Abstract

### Hit the Ground Running: Wasm Snapshots for Fast Start Up

Don't make your users wait while your Wasm module initializes itself!
[Wizer](https://github.com/bytecodealliance/wizer) instantiates your WebAssembly
module, executes its initialization functions, and then snapshots the
initialized state out into a new, pre-initialized WebAssembly module. Now you
can use this new module to hit the ground running, without waiting for any of
that first-time initialization code to complete. This talk will cover the design
and implementation of Wizer; discuss its performance characteristics and the
scenarios in which it excels and when it isn't the right tool; and finally, in
the process of doing all that, we'll take a closer look at what makes up the
guts of a WebAssembly module: memories, globals, tables, etc.

## Slides

[**View the slide deck here!**](https://docs.google.com/presentation/d/1DezYcZ2mPBUN6yW7ZiAqBX_KRr0lLS3NM1RDH5lAfoI/)

## Recording

TODO
