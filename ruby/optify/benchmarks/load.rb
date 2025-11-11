# typed: strict
# frozen_string_literal: true

require 'benchmark'
require_relative '../lib/optify'

puts "PID: #{Process.pid}"

N = 1000

Benchmark.bm(10) do |x|
  x.report('add_directory') do
    N.times do
      builder = Optify::OptionsProviderBuilder.new
      builder.add_directory('../../tests/test_suites/simple/configs')
    end
  end
end
