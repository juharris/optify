#!/usr/bin/env ruby
# typed: false
# frozen_string_literal: true

# Run a single benchmark with optify2's pure Ruby implementation

require 'pathname'

benchmark_file = ARGV[0]

unless benchmark_file
  warn 'Usage: run_single.rb <benchmark_file>'
  exit 1
end

unless File.exist?(benchmark_file)
  warn "Benchmark file not found: #{benchmark_file}"
  exit 1
end

# Set up paths
OPTIFY2_ROOT = Pathname.new(__dir__).parent
OPTIFY_ROOT = OPTIFY2_ROOT.parent.join('optify')

# Pre-load optify2's implementation
$LOAD_PATH.unshift(OPTIFY2_ROOT.join('lib').to_s)
require 'optify'

# Register optify as loaded from the path the benchmarks will use
optify_path_from_benchmarks = OPTIFY_ROOT.join('lib', 'optify.rb').expand_path.to_s
$LOADED_FEATURES << optify_path_from_benchmarks unless $LOADED_FEATURES.include?(optify_path_from_benchmarks)

# Run the benchmark
load benchmark_file
