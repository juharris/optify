#!/bin/bash
set -e

# Go to the root directory of the project.
cd "$(dirname "$0")/.."

pushd rust/optify
cargo test
popd

pushd js/optify-config
yarn build
yarn test
popd

pushd python/optify
maturin develop
pytest
popd

pushd ruby/optify
rake test