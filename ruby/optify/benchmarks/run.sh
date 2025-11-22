#!/bin/bash
set -e

# Go to the root directory of the Ruby project.
cd "$(dirname "$0")/.."

rm -rf target/ tmp/ pkg/ ext/optify_ruby/target/

RB_SYS_CARGO_PROFILE='release' rake native gem

echo "Running benchmarks..."
for benchmark_file in benchmarks/*.rb; do
  echo
  echo "Running $benchmark_file"
  ruby $benchmark_file
done

