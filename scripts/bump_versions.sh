#!/bin/bash
set -e

# major, minor, or patch
strategy=$1


# Go to the root directory of the project.
cd "$(dirname "$0")/.."

# TODO Bump in Cargo.toml files.
# Run `cargo build`.
# Bump in each language's file.

pushd rust/optify
popd

pushd python/optify
# TODO bump in pyproject.toml
popd

pushd ruby/optify
# TODO bump in optify.gemspec
popd

pushd js/optify-config
yarn version $strategy
yarn install
popd

# Don't do the extension because it needs @optify/config to be published first.