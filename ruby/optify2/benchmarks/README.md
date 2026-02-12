# Benchmarks

This directory contains scripts to run the shared benchmarks located in `../../optify/benchmarks/` using the this pure Ruby implementation.

## Philosophy

Benchmarks are maintained in **one location** to avoid duplication and ensure consistency. The runner script here uses Ruby's `$LOADED_FEATURES` mechanism to elegantly substitute optify2's implementation when the benchmark files are executed.

## How It Works

The `run` script:
1. Pre-loads optify2's implementation
2. Registers it in `$LOADED_FEATURES` at the path where benchmarks expect it
3. When benchmarks do `require_relative '../lib/optify'`, Ruby sees it's already loaded and skips it
4. Result: Same benchmark files, different implementation - no duplication!

## Usage

```bash
# Run benchmarks with optify2
./benchmarks/run

# Compare both implementations
./benchmarks/compare
```
