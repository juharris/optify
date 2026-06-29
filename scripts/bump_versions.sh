#!/bin/bash
set -e

# Run as `bash --login ./scripts/bump_versions.sh <strategy>` to automatically get the right environments for Node and other managers.

# major, minor, or patch
strategy=$1

sed_replace_in_place() {
    local file="$1"
    shift
    sed -i.bak "$@" "$file"
    rm -f "${file}.bak"
}

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
    sed_replace_in_place "$file" "s/^version = \"${current_version}\"/version = \"${next_version}\"/"
}

bump_version_gemspec() {
    local file="$1"
    local strategy="$2"
    local current_version=$(grep -m 1 "^VERSION = '" $file | sed -E "s/VERSION = '(.+)'/\1/")
    local next_version=$(get_next_version $current_version $strategy)
    sed_replace_in_place "$file" "s/^VERSION = '${current_version}'/VERSION = '${next_version}'/"
}

bump_dependency_in_toml() {
    local file="$1"
    local current_version=$2
    local next_version=$3
    sed_replace_in_place "$file" -E "s/^(optify = \\{ path = \".+\", version = \")${current_version}(\" \\}$)/\\1${next_version}\\2/"
    # Handle `optify = "x.y.z"` case as well
    sed_replace_in_place "$file" -E "s/^(optify = \")${current_version}(\"$)/\\1${next_version}\\2/"
}

# Go to the root directory of the project.
cd "$(dirname "$0")/.."

pushd rust/optify
current_version=$(grep -m 1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
next_version=$(get_next_version $current_version $strategy)
bump_version_in_toml "Cargo.toml" $strategy
popd

pushd rust/optify-cli
bump_version_in_toml "Cargo.toml" $strategy
bump_dependency_in_toml "Cargo.toml" $current_version $next_version
popd

pushd elixir/optify
bump_dependency_in_toml "native/optify_nif/Cargo.toml" $current_version $next_version
bump_version_in_toml "native/optify_nif/Cargo.toml" $strategy
# Bump version in mix.exs
current_elixir_version=$(sed -nE 's/^[[:space:]]*@version[[:space:]]+"(.+)".*/\1/p' mix.exs | head -n 1)
if [[ -z "$current_elixir_version" ]]; then
    current_elixir_version=$(sed -nE 's/.*version:[[:space:]]+"(.+)".*/\1/p' mix.exs | head -n 1)
fi
if [[ -z "$current_elixir_version" ]]; then
    echo "Unable to determine version from mix.exs. Expected @version \"x.y.z\" or version: \"x.y.z\"." >&2
    exit 1
fi
next_elixir_version=$(get_next_version $current_elixir_version $strategy)
if grep -Eq '^[[:space:]]*@version[[:space:]]+"' mix.exs; then
    sed_replace_in_place mix.exs -E "s/@version[[:space:]]+\"${current_elixir_version}\"/@version \"${next_elixir_version}\"/"
else
    sed_replace_in_place mix.exs "s/version: \"${current_elixir_version}\"/version: \"${next_elixir_version}\"/"
fi
# Update Cargo.lock in the Elixir native crate.
cargo check
popd

pushd python/optify
bump_dependency_in_toml "Cargo.toml" $current_version $next_version
bump_version_in_toml "pyproject.toml" $strategy
bump_version_in_toml "Cargo.toml" $strategy
cargo check
popd

pushd ruby/optify
bump_dependency_in_toml "ext/optify_ruby/Cargo.toml" $current_version $next_version
bump_version_in_toml "ext/optify_ruby/Cargo.toml" $strategy
bump_version_gemspec "optify.gemspec" $strategy
# Update Gemfile.lock
bundle install
popd

pushd js/optify-config
bump_dependency_in_toml "Cargo.toml" $current_version $next_version
yarn version $strategy
yarn install
popd

# Don't do the extension because it needs @optify/config to be published first.
