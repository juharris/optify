# typed: true
# frozen_string_literal: true

require 'test/unit'
require 'optify'
require_relative 'my_config'

class AdditionalFeaturesTest < Test::Unit::TestCase
  BUILDERS = [Optify::OptionsProviderBuilder, Optify::OptionsWatcherBuilder].freeze
  PROVIDERS = [Optify::OptionsProvider, Optify::OptionsWatcher].freeze

  def test_get_dependents
    PROVIDERS.each do |klass|
      provider = klass.build('../../tests/test_suites/inheritance/configs')

      grandparent_metadata = provider.get_feature_metadata('grandparent') #: as !nil
      assert_not_nil(grandparent_metadata)
      assert_nil(grandparent_metadata.dependents)

      parent1_metadata = provider.get_feature_metadata('parent1') #: as !nil
      parent1_dependents = parent1_metadata.dependents #: as !nil
      assert_not_nil(parent1_dependents)
      dependents = parent1_dependents.sort
      assert_equal(%w[grandparent grandparent_too], dependents)

      base1_metadata = provider.get_feature_metadata('base1') #: as !nil
      base1_dependents = base1_metadata.dependents #: as !nil
      assert_not_nil(base1_dependents)
      dependents = base1_dependents.sort
      assert_equal(%w[parent1 super super_with_options], dependents)

      base2_metadata = provider.get_feature_metadata('base2') #: as !nil
      base2_dependents = base2_metadata.dependents #: as !nil
      assert_not_nil(base2_dependents)
      dependents = base2_dependents.sort
      assert_equal(%w[parent2 super super_with_options], dependents)
    end
  end

  def test_provider_get_options_with_overrides_complex
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
      preferences = Optify::GetOptionsPreferences.new
      preferences.overrides = {
        'myConfig' => {
          'new key' => 33,
          'rootString' => 'new string',
          'myObject' => {
            'one' => 1321,
            'something new for test_provider_get_options_with_overrides' => 'hello',
          },
        },
      }

      opts_json = provider.get_options_json_with_preferences('myConfig', ['a'], preferences)
      opts = JSON.parse(opts_json)

      assert_equal(33, opts['new key'])
      assert_equal('new string', opts['rootString'])
      assert_equal('gets overridden', opts['rootString2'])
      assert_equal(1321, opts['myObject']['one'])
      assert_equal(2, opts['myObject']['two'])
      assert_equal('hello', opts['myObject']['something new for test_provider_get_options_with_overrides'])
      assert_equal('string', opts['myObject']['string'])
    end
  end

  def test_configurable_values_get_all_options_with_overrides
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/configurable_values/configs')
                      .build
      preferences = Optify::GetOptionsPreferences.new
      preferences.enable_configurable_strings
      preferences.overrides = {
        'message' => {
          '$type' => 'Optify.ConfigurableString',
          'base' => {
            'liquid' => 'Hello {{ name }}!',
          },
          'arguments' => {
            'name' => 'from the test',
          },
        },
      }

      features = []
      opts_json = provider.get_all_options_json(features, preferences)
      opts = JSON.parse(opts_json)

      assert_equal('Hello from the test!', opts['message'])
    end
  end

  def test_empty_features_array
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
      features = []
      # With empty features, get_all_options should work
      all_opts_json = provider.get_all_options_json(features, Optify::GetOptionsPreferences.new)
      all_opts = JSON.parse(all_opts_json)
      assert_not_nil(all_opts)
      assert_kind_of(Hash, all_opts)
    end
  end

  def test_build_from_multiple_directories
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/configurable_values/configs')
                      .add_directory('../../tests/test_suites/inheritance/configs')
                      .build
      # Should have features from both directories
      features = provider.features
      assert_includes(features, 'simple')
      assert_includes(features, 'grandparent')
      assert_includes(features, 'base1')
    end
  end

  def test_get_feature_metadata_for_nonexistent_feature
    PROVIDERS.each do |klass|
      provider = klass.build('../../tests/test_suites/simple/configs')
      metadata = provider.get_feature_metadata('nonexistent_feature')
      assert_nil(metadata)
    end
  end

  def test_skip_feature_name_conversion_with_invalid_name
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
      preferences = Optify::GetOptionsPreferences.new
      preferences.skip_feature_name_conversion = true

      err = assert_raise(Optify::UnknownFeatureError) do
        provider.get_options('myConfig', ['a'], MyConfig, nil, preferences)
      end
      assert_match(/Feature name "a" is not a known feature/, err.message)

      err = assert_raise(Optify::UnknownFeatureError) do
        provider.get_options('myConfig', ['A'], MyConfig, nil, preferences)
      end
      assert_match(/Feature name "A" is not a known feature/, err.message)
    end
  end

  def test_get_all_options_with_empty_features
    BUILDERS.each do |klass|
      provider = klass.new
                      .add_directory('../../tests/test_suites/simple/configs')
                      .build
      features = []
      all_opts_json = provider.get_all_options_json(features, Optify::GetOptionsPreferences.new)
      all_opts = JSON.parse(all_opts_json)
      assert_not_nil(all_opts)
      assert_kind_of(Hash, all_opts)
      # With empty features, the config should still be available if there's a base config
      # but it may be empty depending on how the system handles it
      # So we just check that we get a hash back
    end
  end

  def test_features_with_metadata_path
    PROVIDERS.each do |klass|
      provider = klass.build('../../tests/test_suites/simple/configs')
      metadata = provider.features_with_metadata
      a_metadata = metadata['feature_A'] #: as !nil
      assert_not_nil(a_metadata)
      assert_not_nil(a_metadata.path)
      assert_match(/feature_A\.json/, a_metadata.path)
    end
  end
end
