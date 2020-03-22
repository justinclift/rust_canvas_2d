#!/bin/sh

set -ex

wasm-pack build --target no-modules --no-typescript
go run server.go
