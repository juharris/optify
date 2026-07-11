#!/bin/bash
set -e

# Go to the root directory of the project.
cd "$(dirname "$0")/../../.."

source ./scripts/version_utils.sh

# major, minor, or patch
strategy=$1

pushd ruby/from_hash
current_version=$(get_current_gemspec_version "optify-from_hash.gemspec")
bump_version_gemspec "optify-from_hash.gemspec" $strategy
bundle install
popd

pushd ruby/optify
# Bump dependency version in optify.gemspec
next_version=$(get_next_version $current_version $strategy)
sed_replace_in_place "optify.gemspec" -E "s/^(FROM_HASH_DEP_VERSION = ')${current_version}(')/\\1${next_version}\\2/"

# Update Gemfile.lock
bundle install
bundle exec tapioca gem optify-from_hash
popd