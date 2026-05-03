#!/bin/bash
set -e

# Go to the root directory of the project.
cd "$(dirname "$0")/.."

pushd rust
cargo test
popd

pushd elixir/optify
mix test
popd

pushd js/optify-config
yarn build:debug
yarn test
popd

pushd python/optify
maturin develop
pytest
popd

pushd ruby/optify
bundle exec rake test