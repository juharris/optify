# frozen_string_literal: true
# typed: true

require 'test/unit'
require_relative '../lib/optify'
require_relative 'conditions_config'
require_relative 'my_config'

class ProviderCacheTest < Test::Unit::TestCase
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

    preferences.constraints = { info: 3, status: 'active' }
    config_a_b2 = provider.get_options('config', %w[a b], MyConditionsConfig, cache_options, preferences)
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
  end
end
