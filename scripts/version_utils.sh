#!/bin/bash
set -e

# Methods to share with scripts to bump versions.

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

get_current_gemspec_version() {
    local file="$1"
    grep -Em 1 "^(FROM_HASH|OPTIFY)_VERSION = '" $file | sed -E "s/^.*VERSION = '(.+)'/\1/"
}

bump_version_gemspec() {
    local file="$1"
    local strategy="$2"
    local current_version=$(get_current_gemspec_version $file)
    local next_version=$(get_next_version $current_version $strategy)
    sed_replace_in_place "$file" "s/VERSION = '${current_version}'/VERSION = '${next_version}'/"
}