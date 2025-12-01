# typed: strict
# frozen_string_literal: true

require 'benchmark'
require_relative '../lib/optify'
require_relative '../test/my_config'

puts "PID: #{Process.pid}"

N = 1000
WARMUP_COUNT = 22

# Simple test suite
simple_provider = Optify::OptionsProvider.build('../../tests/test_suites/simple/configs')

feature_a = simple_provider.get_canonical_feature_name('a')
feature_b = simple_provider.get_canonical_feature_name('b')

simple_feature_trials = [
  [feature_a],
  ['a', feature_a, 'b', feature_b],
  ['a', feature_a, 'b', feature_b, 'A_with_comments', 'a', 'B', 'a', feature_a, 'b', feature_b, 'A_with_comments', 'a', 'B', 'a', feature_a, 'b', feature_b, 'A_with_comments',
   'a', 'B']
]

# Configurable values test suite
configurable_provider = Optify::OptionsProvider.build('../../tests/test_suites/configurable_values/configs')

configurable_preferences = Optify::GetOptionsPreferences.new
configurable_preferences.enable_configurable_strings

configurable_feature_trials = [
  ['simple'],
  %w[simple imports],
  ['imports_imports'],
  %w[simple override_name],
  %w[simple raw_overrides],
  %w[simple with_files],
  %w[simple with_files_in_arguments],
  %w[simple complex_deep_merge],
  %w[simple complex_wide_structure],
  %w[simple complex_deep_merge complex_nested_objects complex_wide_structure]
]

Benchmark.bm do |x|
  x.report('Simple test suite:') do
    # Empty report for grouping
  end

  simple_feature_trials.each do |features|
    WARMUP_COUNT.times do
      simple_provider.get_options_json('myConfig', features)
    end

    WARMUP_COUNT.times do
      simple_provider.get_options_hash('myConfig', features)
    end

    x.report("  get_options_hash 'myConfig' (features: #{features.join(', ')})") do
      N.times do
        _options = simple_provider.get_options_hash('myConfig', features)
      end
    end

    WARMUP_COUNT.times do
      simple_provider.get_options('myConfig', features, MyConfig)
    end

    x.report("  get_options 'myConfig' (features: #{features.join(', ')})") do
      N.times do
        _options = simple_provider.get_options('myConfig', features, MyConfig)
      end
    end
  end
end

Benchmark.bm do |x|
  x.report('Configurable values test suite:') do
    # Empty report for grouping
  end

  configurable_feature_trials.each do |features|
    WARMUP_COUNT.times do
      configurable_provider.get_all_options_hash(features, configurable_preferences)
    end

    x.report("  get_all_options_hash (features: #{features.join(', ')})") do
      N.times do
        _options = configurable_provider.get_all_options_hash(features, configurable_preferences)
      end
    end
  end
end
