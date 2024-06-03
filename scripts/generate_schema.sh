#!/bin/bash

set -xe

TARGET_DIR=$(pwd)/target/schemas
CLIENT_DIR=$(pwd)/apps/client/schemas/api.d.ts

mkdir -p $TARGET_DIR
cargo run -p generator $TARGET_DIR/api.json
pnpm -F @chesu/client exec openapi-typescript $TARGET_DIR/api.json -o $CLIENT_DIR
