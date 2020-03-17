#!/bin/sh

set -ex

wasm-pack build --target web --no-typescript
go run server.go
