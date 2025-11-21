# typed: strict
# frozen_string_literal: true

require 'benchmark'
require_relative '../lib/optify'

puts "PID: #{Process.pid}"

N = 1000
WARMUP_COUNT = 10

Benchmark.bm(10) do |x|
  path = '../../tests/test_suites/simple/configs'
  WARMUP_COUNT.times do
    builder = Optify::OptionsProviderBuilder.new
    builder.add_directory(path)
  end

  x.report('add_directory') do
    N.times do
      builder = Optify::OptionsProviderBuilder.new
      builder.add_directory(path)
    end
  end
end
