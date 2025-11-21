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
    _provider = Optify::OptionsProvider.build(path)
  end

  x.report('build simple') do
    N.times do
      _provider = Optify::OptionsProvider.build(path)
    end
  end

  path = '../../tests/test_suites/inheritance/configs'
  x.report('build inheritance') do
    N.times do
      _provider = Optify::OptionsProvider.build(path)
    end
  end

  path = '../../tests/test_suites/configurable_values/configs'
  x.report('build configurable values') do
    N.times do
      _provider = Optify::OptionsProvider.build(path)
    end
  end
end
