# frozen_string_literal: true
# typed: true

require 'json'
require 'test/unit'
require_relative '../lib/optify'
require_relative 'my_config'

class OptifyTest < Test::Unit::TestCase
  BUILDERS = [Optify::OptionsProviderBuilder, Optify::OptionsWatcherBuilder].freeze
  PROVIDERS = [Optify::OptionsProvider, Optify::OptionsWatcher].freeze

  def test_empty_build
    BUILDERS.each do |klass|
      builder = klass.new
      provider = builder.build
      assert_not_nil(provider, "Failed to build #{klass}")
      assert_equal([], provider.aliases.sort!)
      assert_equal([], provider.features.sort!)
      assert_equal([], provider.features_and_aliases.sort!)
    end
  end

  #: (String suite_path) -> void
  def run_suite(suite_path)
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory(File.join(suite_path, 'configs'))
                      .build
      expectations_path = File.join(suite_path, 'expectations')
      Dir.each_child(expectations_path) do |test_case|
        expectation_path = File.join(expectations_path, test_case)
        expected_info = JSON.parse(File.read(expectation_path))
        expected_options = expected_info['options']
        features = expected_info['features']
        constraints = expected_info['constraints']
        preferences = Optify::GetOptionsPreferences.new
        preferences.constraints = constraints
        expected_options.each do |key, expected_value|
          expected_json = provider.get_options_json_with_preferences(key, features, preferences)
          options = JSON.parse(expected_json, object_class: Hash)
          expected_json = expected_value.to_json
          expected_open_struct = JSON.parse(expected_json, object_class: Hash)
          assert_equal(expected_open_struct, options,
                       "Options for key \"#{key}\" with features #{features}
                     do not match for test suite at #{expectation_path}")
        end
      end
    end
  end

  def test_aliases
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
      aliases = provider.aliases.sort!
      assert_equal(%w[a b], aliases)
    end
  end

  def test_get_all_options
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
      features = ['a']
      all_opts = provider.get_all_options_json(features, Optify::GetOptionsPreferences.new)
      key = 'myConfig'
      opts = provider.get_options_json(key, features)
      expected = { key => JSON.parse(opts) }
      assert_equal(expected, JSON.parse(all_opts))
    end
  end

  def test_get_canonical_feature_names
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
      feature_names = %w[A B feature_A]
      canonical_feature_names = provider.get_canonical_feature_names(feature_names)
      assert_equal(%w[feature_A feature_B/initial feature_A], canonical_feature_names)

      feature_names = %w[A B feature_A a b feature_B/initial a B fEaTuRe_A]
      canonical_feature_names = provider.get_canonical_feature_names(feature_names)
      assert_equal(%w[feature_A feature_B/initial feature_A feature_A feature_B/initial feature_B/initial feature_A feature_B/initial feature_A], canonical_feature_names)

      err = assert_raise do
        provider.get_canonical_feature_names(%w[error])
      end
      assert_equal('Feature name "error" is not a known feature.', err.message)

      names = feature_names.map { |name| provider.get_canonical_feature_name(name) }
      assert_equal(canonical_feature_names, names)
    end
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
    BUILDERS.each do |klass|
      provider = klass.new
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
  end

  def test_cache_with_preferences
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
                      .init
      cache_options = Optify::CacheOptions.new
      preferences = Optify::GetOptionsPreferences.new
      preferences.skip_feature_name_conversion = false
      config_a = provider.get_options('myConfig', ['A'], MyConfig, cache_options, preferences)
      assert_false(preferences.skip_feature_name_conversion)
      config_a2 = provider.get_options('myConfig', ['a'], MyConfig, cache_options, preferences)
      assert_false(preferences.skip_feature_name_conversion)
      assert_same(config_a, config_a2)
    end
  end

  def test_custom_config_class
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
      config = provider.get_options('myConfig', ['A'], MyConfig)
      assert_equal('root string same', config.rootString)
      assert_equal(['example item 1'], config.myArray)
      assert_equal(2, config.myObject.two)
    end
  end

  def test_features
    PROVIDERS.each do |klass|
      provider = klass.build('../../tests/test_suites/simple/configs')
      all_features = provider.features
      all_features.sort!
      assert_equal(['A_with_comments', 'feature_A', 'feature_B/initial'], all_features)
    end
  end

  def test_features_and_aliases
    PROVIDERS.each do |klass|
      provider = klass.build_from_directories(['../../tests/test_suites/simple/configs'])
      features_and_aliases = provider.features_and_aliases.sort!
      assert_equal(
        ['A_with_comments',
         'a',
         'b',
         'feature_A',
         'feature_B/initial'],
        features_and_aliases
      )
    end
  end

  def test_get_options_with_cache_and_overrides
    PROVIDERS.each do |klass|
      provider = klass.build('../../tests/test_suites/simple/configs')
                      .init
      cache_options = Optify::CacheOptions.new
      feature_names = %w[A B]
      preferences = Optify::GetOptionsPreferences.new
      preferences.overrides = {}
      exception = assert_raise(ArgumentError) do
        provider.get_options('myConfig', feature_names, MyConfig, cache_options, preferences)
      end
      assert_equal('Caching when overrides are given is not supported. Do not pass cache options when using overrides in preferences.', exception.message)
    end
  end

  def test_get_options_with_overrides
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
      cache_options = nil
      feature_names = %w[A B]
      preferences = Optify::GetOptionsPreferences.new
      options_without_overrides = provider.get_options('myConfig', feature_names, MyConfig, cache_options, preferences)
      assert_equal('root string same', options_without_overrides.rootString)

      preferences = Optify::GetOptionsPreferences.new
      preferences.overrides = { 'myConfig' => { rootString: 'root string overrides' } }
      options = provider.get_options('myConfig', feature_names, MyConfig, cache_options, preferences)
      assert_equal('root string overrides', options.rootString)
      assert_equal(options_without_overrides.rootString2, options.rootString2)

      # Test that overrides work for nested values.
      value = 2222
      assert_not_equal(value, options_without_overrides.myObject.two)
      preferences.overrides = { 'myConfig' => { 'myObject' => { 'two' => value } } }
      options = provider.get_options('myConfig', feature_names, MyConfig, cache_options, preferences)
      assert_equal(options_without_overrides.myObject.one, options.myObject.one)
      assert_equal(value, options.myObject.two)
      assert_equal(options_without_overrides.rootString, options.rootString)
      assert_equal(options_without_overrides.rootString2, options.rootString2)
    end
  end

  def test_get_options_with_preferences
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
      feature_names = %w[A B]
      preferences = Optify::GetOptionsPreferences.new
      preferences.overrides = nil
      preferences.skip_feature_name_conversion = false
      options = provider.get_options('myConfig', feature_names, MyConfig, nil, preferences)
      assert_equal('root string same', options.rootString)
      s = provider.get_options_json('myConfig.rootString', feature_names)
      assert_equal('"root string same"', s)
      assert_equal('root string same', JSON.parse(s))
      s = provider.get_options_json('myConfig.myObject.two', feature_names)
      assert_equal('22', s)
      assert_equal(22, JSON.parse(s))

      preferences.skip_feature_name_conversion = true
      err = assert_raise(RuntimeError) do
        provider.get_options('myConfig', feature_names, MyConfig, nil, preferences)
      # Ensure that consumers of this library can use `rescue => e`.
      rescue => e # rubocop:disable Style/RescueStandardError
        # Expected error.
        raise e
      rescue Exception => e # rubocop:disable Style/RescueException
        flunk "Expected RuntimeError that can be caught by `rescue => e`, got #{e.class}"
      end
      assert_equal('Feature name "A" is not a known feature.', err.message)
    end
  end
end
