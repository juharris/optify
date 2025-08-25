#!/bin/bash
set -e

# major, minor, or patch
strategy=$1

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

bump_version_in_toml() {
    local file="$1"
    local strategy="$2"
    local current_version=$(grep -m 1 '^version = ' $file | sed -E 's/version = "(.+)"/\1/')
    local next_version=$(get_next_version $current_version $strategy)
    sed -i "" 's/^version = "'$current_version'"/version = "'$next_version'"/' "$file"
}

bump_version_gemspec() {
    local file="$1"
    local strategy="$2"
    local current_version=$(grep -m 1 "^VERSION = '" $file | sed -E "s/VERSION = '(.+)'/\1/")
    local next_version=$(get_next_version $current_version $strategy)
    sed -i "" "s/^VERSION = '${current_version}'/VERSION = '${next_version}'/" "$file"
}

bump_dependency_in_toml() {
    local file="$1"
    local current_version=$2
    local next_version=$3
    sed -i "" -E 's/^(optify = \{ path = ".+", version = ")'${current_version}'(" \}$)/\1'${next_version}'\2/' "$file"
}

# Go to the root directory of the project.
cd "$(dirname "$0")/.."

pushd rust/optify
current_version=$(grep -m 1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
next_version=$(get_next_version $current_version $strategy)
bump_version_in_toml "Cargo.toml" $strategy
popd

pushd python/optify
bump_dependency_in_toml "Cargo.toml" $current_version $next_version
bump_version_in_toml "pyproject.toml" $strategy
bump_version_in_toml "Cargo.toml" $strategy
maturin build
popd

pushd ruby/optify
bump_dependency_in_toml "ext/optify_ruby/Cargo.toml" $current_version $next_version
bump_version_in_toml "ext/optify_ruby/Cargo.toml" $strategy
bump_version_gemspec "optify.gemspec" $strategy
popd

pushd js/optify-config
bump_dependency_in_toml "Cargo.toml" $current_version $next_version
yarn version $strategy
yarn install
popd

# Don't do the extension because it needs @optify/config to be published first.
