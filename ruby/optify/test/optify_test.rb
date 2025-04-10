# frozen_string_literal: true
# typed: true

require 'json'
require 'test/unit'
require_relative '../lib/optify'
require_relative 'my_config'

class OptifyTest < Test::Unit::TestCase
  def test_empty_build
    builder = Optify::OptionsProviderBuilder.new
    provider = builder.build
    assert_not_nil(provider)
  end

  #: (String suite_path) -> void
  def run_suite(suite_path)
    provider = Optify::OptionsProviderBuilder.new
                                             .add_directory(File.join(suite_path, 'configs'))
                                             .build
    expectations_path = File.join(suite_path, 'expectations')
    Dir.each_child(expectations_path) do |test_case|
      expectation_path = File.join(expectations_path, test_case)
      expected_info = JSON.parse(File.read(expectation_path))
      expected_options = expected_info['options']
      features = expected_info['features']
      expected_options.each do |key, expected_value|
        expected_json = provider.get_options_json(key, features)
        options = JSON.parse(expected_json, object_class: Hash)
        expected_json = expected_value.to_json
        expected_open_struct = JSON.parse(expected_json, object_class: Hash)
        assert_equal(expected_open_struct, options,
                     "Options for key \"#{key}\" with features #{features}
                     do not match for test suite at #{expectation_path}")
      end
    end
  end

  def test_get_all_options
    provider = Optify::OptionsProviderBuilder.new
                                             .add_directory('../../tests/test_suites/simple/configs')
                                             .build
    features = ['a']
    all_opts = provider.get_all_options_json(features, Optify::GetOptionsPreferences.new)
    key = 'myConfig'
    opts = provider.get_options_json(key, features)
    expected = { key => JSON.parse(opts) }
    assert_equal(expected, JSON.parse(all_opts))
  end

  def test_get_canonical_feature_names
    provider = Optify::OptionsProviderBuilder.new
                                             .add_directory('../../tests/test_suites/simple/configs')
                                             .build
    canonical_feature_names = provider.get_canonical_feature_names(%w[A B feature_A])
    assert_equal(%w[feature_A feature_B/initial feature_A], canonical_feature_names)

    err = assert_raise do
      provider.get_canonical_feature_names(%w[error])
    end
    assert_equal('given names should be valid: "The given feature \"error\" was not found."', err.message)
  end

  def test_suites
    test_suites_dir = '../../tests/test_suites'
    Dir.each_child(test_suites_dir) do |suite|
      suite_path = File.join(test_suites_dir, suite)
      next unless File.directory?(suite_path)

      run_suite(suite_path)
    end
  end

  def test_cache
    provider = Optify::OptionsProviderBuilder.new
                                             .add_directory('../../tests/test_suites/simple/configs')
                                             .build
                                             .init
    cache_options = Optify::CacheOptions.new
    config_a = provider.get_options('myConfig', ['A'], MyConfig, cache_options)
    config_b = provider.get_options('myConfig', ['B'], MyConfig, cache_options)
    config_b2 = provider.get_options('myConfig', ['B'], MyConfig, cache_options)
    config_b3 = provider.get_options('myConfig', ['b'], MyConfig, cache_options)
    config_b4 = provider.get_options('myConfig', ['featUre_B/iNITial'], MyConfig, cache_options)
    assert_not_same(config_a, config_b)
    assert_same(config_b, config_b2)
    assert_same(config_b, config_b3)
    assert_same(config_b, config_b4)

    config_a_b = provider.get_options('myConfig', %w[A B], MyConfig, cache_options)
    config_b_a = provider.get_options('myConfig', %w[B A], MyConfig, cache_options)
    assert_not_same(config_a_b, config_b_a)
    config_a_b2 = provider.get_options('myConfig', ['A', 'featUre_B/iNITial'], MyConfig, cache_options)
    assert_same(config_a_b, config_a_b2)
  end

  def test_custom_config_class
    provider = Optify::OptionsProviderBuilder.new
                                             .add_directory('../../tests/test_suites/simple/configs')
                                             .build
    config = provider.get_options('myConfig', ['A'], MyConfig)
    assert_equal('root string same', config.rootString)
    assert_equal(['example item 1'], config.myArray)
    assert_equal(2, config.myObject.two)
  end

  def test_features
    provider = Optify::OptionsProviderBuilder.new
                                             .add_directory('../../tests/test_suites/simple/configs')
                                             .build
    all_features = provider.features
    all_features.sort!
    assert_equal(['A_with_comments', 'feature_A', 'feature_B/initial'], all_features)
  end
end
