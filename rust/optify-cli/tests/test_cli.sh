#!/usr/bin/env bash
# Integration tests for the optify CLI binary.
# Installs the binary via cargo and then invokes it as `optify`.
# Run from the rust/optify-cli directory.

set -euo pipefail

CONFIGS="../../tests/test_suites/simple/configs"

echo "Installing optify CLI..."
cargo install --path . --quiet
echo "Installed $(optify --version)"

pass=0
fail=0

check() {
    local description="$1"
    local expected="$2"
    local actual="$3"
    if [ "$actual" = "$expected" ]; then
        echo "PASS: $description"
        (( ++pass ))
    else
        echo "FAIL: $description"
        echo "  Expected: $expected"
        echo "  Actual:   $actual"
        (( ++fail ))
    fi
}

# list-features returns a JSON array of metadata
# delete the "path" field since it's not deterministic and sort by name for consistent ordering
check "list-features" \
    '[{"aliases":null,"dependents":null,"details":null,"name":"A_with_comments","owners":"a-team@company.com"},{"aliases":["a"],"dependents":null,"details":"The file is for testing.","name":"feature_A","owners":"a-team@company.com"},{"aliases":["b"],"dependents":null,"details":{"description":"This is a description of the feature."},"name":"feature_B/initial","owners":"team-b@company.com"}]' \
    "$(optify --dir "$CONFIGS" list-features | jq -c '[.[] | del(.path)] | sort_by(.name)')"

# get-options outputs compact (single-line) JSON
check "get-options -k myConfig -f A" \
    '{"myArray":["example item 1"],"myObject":{"deeper":{"list":[1,2],"wtv":3},"one":1,"string":"string","two":2},"rootString":"root string same","rootString2":"gets overridden"}' \
    "$(optify --dir "$CONFIGS" get-options -k myConfig -f A)"

# get-options with multiple features — later feature overrides earlier
check "get-options -k myConfig -f A B" \
    '{"myArray":["different item 1","item 2"],"myObject":{"deeper":{"list":[55],"new":"new value","wtv":3333},"one":1,"string":"string","three":3,"two":22},"rootString":"root string same","rootString2":"override"}' \
    "$(optify --dir "$CONFIGS" get-options -k myConfig -f A B)"

# get-all-options returns the full merged configuration
check "get-all-options -f A" \
    '{"myConfig":{"myArray":["example item 1"],"myObject":{"deeper":{"list":[1,2],"wtv":3},"one":1,"string":"string","two":2},"rootString":"root string same","rootString2":"gets overridden"}}' \
    "$(optify --dir "$CONFIGS" get-all-options -f A)"

# error on unknown feature exits non-zero and prints to stderr
if optify --dir "$CONFIGS" get-options -k myConfig -f unknown_feature 2>/dev/null; then
    echo "FAIL: unknown feature should exit non-zero"
    (( ++fail ))
else
    echo "PASS: unknown feature exits non-zero"
    (( ++pass ))
fi

echo ""
if [ "$fail" -ne 0 ]; then
    echo "$fail test(s) failed."
    exit 1
fi
echo "All $pass tests passed."
