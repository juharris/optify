# frozen_string_literal: true
# typed: true

require 'test/unit'
require_relative '../lib/optify'
require_relative 'conditions_config'
require_relative 'my_config'

class ProviderCacheTest < Test::Unit::TestCase
  class StringConfig < Optify::BaseConfig
    def self.from_hash(obj)
      obj.is_a?(String) ? obj : super(obj)
    end
  end

  def test_cache_with_configurable_strings
    provider = Optify::OptionsProvider
               .build('../../tests/test_suites/simple/configs')
               .init
    cache_options = Optify::CacheOptions.new
    preferences = Optify::GetOptionsPreferences.new
    assert !preferences.are_configurable_strings_enabled?
    preferences.enable_configurable_strings
    assert preferences.are_configurable_strings_enabled?
    preferences.disable_configurable_strings
    assert !preferences.are_configurable_strings_enabled?

    config_a = provider.get_options('myConfig', ['A'], MyConfig, cache_options, preferences)
    config_a2 = provider.get_options('myConfig', ['a'], MyConfig, cache_options, nil)
    assert_same(config_a, config_a2)

    preferences.enable_configurable_strings
    assert preferences.are_configurable_strings_enabled?
    config_a_enabled = provider.get_options('myConfig', ['A'], MyConfig, cache_options, preferences)
    assert_not_same(config_a, config_a_enabled)

    config_a2_enabled = provider.get_options('myConfig', ['a'], MyConfig, cache_options, preferences)
    assert_same(config_a_enabled, config_a2_enabled)
  end

  def test_cache_with_constraints
    provider = Optify::OptionsProvider
               .build('../../tests/test_suites/simple/configs')
               .init
    cache_options = Optify::CacheOptions.new
    preferences = Optify::GetOptionsPreferences.new
    preferences.constraints = { wtv: 3 }

    config_a = provider.get_options('myConfig', ['A'], MyConfig, cache_options, preferences)
    assert_equal('root string same', config_a.rootString)
    config_a2 = provider.get_options('myConfig', ['a'], MyConfig, cache_options, preferences)
    assert_same(config_a, config_a2)

    preferences.constraints = { wtv: 4 }
    config_a2 = provider.get_options('myConfig', ['A'], MyConfig, cache_options, preferences)
    assert_same(config_a, config_a2)

    config_b = provider.get_options('myConfig', ['B'], MyConfig, cache_options, preferences)
    assert_not_same(config_a, config_b)
  end

  def test_cache_with_filtered_constraints
    provider = Optify::OptionsProvider
               .build('../../tests/test_suites/conditions/configs')
               .init
    cache_options = Optify::CacheOptions.new
    preferences = Optify::GetOptionsPreferences.new
    # Use constraints that filter out A.
    preferences.constraints = { wtv: 3 }

    config_b = provider.get_options('config', %w[a b], MyConditionsConfig, cache_options, preferences)
    expected = MyConditionsConfig.from_hash({ key: 'from B', key_b: 'only in B' })
    assert_equal(expected, config_b)
    config_b2 = provider.get_options('config', ['b'], MyConditionsConfig, cache_options, preferences)
    assert_same(config_b, config_b2)

    # Constraints match.
    preferences.constraints = { info: 3, status: 'new' }
    config_a_b = provider.get_options('config', %w[a b], MyConditionsConfig, cache_options, preferences)
    expected = MyConditionsConfig.from_hash({ key: 'from B', key_a: 'only in A', key_b: 'only in B' })
    assert_equal(expected, config_a_b)

    # Different constraints, but still match.
    preferences.constraints = { info: 3, status: 'active' }
    config_a_b2 = provider.get_options('config', %w[a b], MyConditionsConfig, cache_options, preferences)
    assert_same(config_a_b, config_a_b2)

    config_a_b2 = provider.get_options('config', %w[a b], MyConditionsConfig, cache_options, nil)
    assert_same(config_a_b, config_a_b2)

    preferences = Optify::GetOptionsPreferences.new
    preferences.skip_feature_name_conversion = true
    config_a_b2 = provider.get_options('config', %w[A B], MyConditionsConfig, cache_options, nil)
    assert_same(config_a_b, config_a_b2)
  end

  def test_cache_with_used_constraints
    provider = Optify::OptionsProvider
               .build('../../tests/test_suites/conditions/configs')
               .init
    cache_options = Optify::CacheOptions.new
    preferences = Optify::GetOptionsPreferences.new
    config = provider.get_options('config', ['docs_example'], MyConditionsConfig, cache_options, preferences)
    expected = MyConditionsConfig.from_hash({ key: 'from docs_example', key_docs_example: 'only in docs_example' })
    assert_equal(expected, config)
    config2 = provider.get_options('config', ['D'], MyConditionsConfig, cache_options, preferences)
    assert_same(config, config2)

    preferences.constraints = { clientId: 1234 }
    config2 = provider.get_options('config', ['docs_example'], MyConditionsConfig, cache_options, preferences)
    assert_equal(config, config2)
    assert_same(config, config2)

    config3 = provider.get_options('config', %w[docs_example B], MyConditionsConfig, cache_options, preferences)
    assert_not_same(config, config3)
  end

  def test_cache_with_configurable_values
    provider = Optify::OptionsProvider
               .build('../../tests/test_suites/configurable_values/configs')
               .init
    cache_options = Optify::CacheOptions.new
    preferences = Optify::GetOptionsPreferences.new
    preferences.enable_configurable_strings

    greeting1 = provider.get_options('greeting', ['simple'], StringConfig, cache_options, preferences)
    assert_equal('Hello, World!', greeting1)

    greeting2 = provider.get_options('greeting', ['simple'], StringConfig, cache_options, preferences)
    assert_same(greeting1, greeting2)

    message1 = provider.get_options('message', ['simple'], StringConfig, cache_options, preferences)
    assert_equal('Welcome to Optify!', message1)
    assert_not_same(greeting1, message1)

    message2 = provider.get_options('message', ['simple'], StringConfig, cache_options, preferences)
    assert_same(message1, message2)
  end
end
