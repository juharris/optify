#!/bin/bash
set -e

# major, minor, or patch
strategy=$1

# Function to get current version and bump it
get_next_version() {
    local current_version=$1
    local strategy=$2
    IFS='.' read -r major minor patch <<< "$current_version"
    
    case $strategy in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "$major.$((minor + 1)).0"
            ;;
        patch)
            echo "$major.$minor.$((patch + 1))"
            ;;
        *)
            echo "Invalid strategy: '$strategy'. Must be 'major', 'minor', or 'patch'." >&2
            exit 1
    esac
}

replace_version_in_toml() {
    local file="$1"
    local current_version=$2
    local next_version=$3
    sed -i "" "s/^version = \"$current_version\"/version = \"$next_version\"/" "$file"
}

# Go to the root directory of the project.
cd "$(dirname "$0")/.."


pushd rust/optify
current_version=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo "Current version: $current_version"
next_version=$(get_next_version $current_version $strategy)
echo "Next version: $next_version"
replace_version_in_toml "Cargo.toml" $current_version $next_version
popd

pushd python/optify
# TODO Bump in Cargo.toml
# TODO Bump dependency in Cargo.toml, like 'optify = { path = "../../rust/optify", version = "0.13.0" }'.
replace_version_in_toml "Cargo.toml" $current_version $next_version
# TODO bump in pyproject.toml (both version = "..." in the 2 sections)
# Bump in Cargo.lock
maturin build
popd

pushd ruby/optify
# TODO bump in optify.gemspec
# TODO bump in ext/optify_ruby/Cargo.toml
# TODO Bump dependency in Cargo.toml
popd

pushd js/optify-config
# TODO Bump dependency in Cargo.toml
yarn version $strategy
yarn install
popd

# Don't do the extension because it needs @optify/config to be published first.
