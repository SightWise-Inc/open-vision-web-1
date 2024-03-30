#!/bin/bash

set -ex

rm -rf public/wasm

cd wasm/process-canvas/
wasm-pack build --release --target web --out-dir ../../public/wasm
cd -
