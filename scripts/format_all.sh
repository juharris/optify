#!/bin/bash
set -e

# Go to the root directory of the project.
cd "$(dirname "$0")/.."

pushd rust/optify
cargo fmt && cargo clippy --fix --allow-dirty --allow-staged
popd

pushd python/optify
maturin develop
cargo fmt && cargo clippy --fix --allow-dirty --allow-staged
popd

pushd ruby/optify
cargo fmt && cargo clippy --fix --allow-dirty --allow-staged