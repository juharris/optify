# typed: strict
# frozen_string_literal: true

require 'benchmark'
require_relative '../lib/optify'

puts "PID: #{Process.pid}"

N = 100_000

builder = Optify::OptionsProviderBuilder.new
builder.add_directory('../../tests/test_suites/simple/configs')
provider = builder.build

feature_a = provider.get_canonical_feature_name('a')
feature_b = provider.get_canonical_feature_name('b')

feature_trials = [
  ['a'],
  [feature_a],
  ['a', feature_a, 'b', feature_b],
  ['a', feature_a, 'b', feature_b, 'A_with_comments', 'a', 'B'],
  ['a', feature_a, 'b', feature_b, 'A_with_comments', 'a', 'B', 'a', feature_a, 'b', feature_b, 'A_with_comments', 'a', 'B', 'a', feature_a, 'b', feature_b, 'A_with_comments',
   'a', 'B']
]

#: (Optify::OptionsProvider, Array[String]) -> Array[String]
def get_canonical_feature_names_loop(provider, feature_names)
  feature_names.map do |feature_name|
    provider.get_canonical_feature_name(feature_name)
  end
end

Benchmark.bm do |x|
  feature_trials.each do |feature_names|
    # Warm up.
    100.times do
      provider.get_canonical_feature_names(feature_names)
    end

    x.report("get_canonical_feature_names (length: #{feature_names.length}) #{feature_names}") do
      N.times do
        _new_names = provider.get_canonical_feature_names(feature_names)
      end
    end

    x.report("get_canonical_feature_name in loop (length: #{feature_names.length}) #{feature_names}") do
      N.times do
        _new_names = get_canonical_feature_names_loop(provider, feature_names)
      end
    end
  end
end
