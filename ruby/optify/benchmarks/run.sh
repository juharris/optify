#!/bin/bash
set -e

# Go to the root directory of the Ruby project.
cd "$(dirname "$0")/.."

rm -rf tmp/

RB_SYS_CARGO_PROFILE='release' RB_SYS_CROSS_COMPILE=true rake native gem

echo "Running benchmarks..."
for benchmark_file in benchmarks/*.rb; do
  echo
  echo "Running $benchmark_file"
  ruby $benchmark_file
done

