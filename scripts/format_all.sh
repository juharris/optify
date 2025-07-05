#!/bin/bash
set -e

# Go to the root directory of the project.
cd "$(dirname "$0")/.."

pushd js/optify-config
cargo fmt
popd

pushd rust/optify
cargo fmt
cargo clippy --fix --allow-dirty --allow-staged
# Verify
cargo clippy --no-deps -- -D warnings --no-deps 
popd

pushd python/optify
maturin develop
cargo fmt
cargo clippy --fix --allow-dirty --allow-staged
# Verify
cargo clippy --no-deps -- -D warnings --no-deps 
popd

pushd ruby/optify
cargo fmt
cargo clippy --fix --allow-dirty --allow-staged
# Verify
cargo clippy --no-deps -- -D warnings --no-deps 
popd

pushd vscode/extension
npm run lint:fix
popd
