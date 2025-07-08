# frozen_string_literal: true
# typed: true

require 'test/unit'
require_relative '../lib/optify'
require_relative 'my_config'

class ProviderCacheTest < Test::Unit::TestCase
  class ExampleConfig < Optify::BaseConfig
    sig { returns(T.nilable(String)) }
    attr_reader :key

    sig { returns(T.nilable(String)) }
    attr_reader :key_docs_example
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
    assert_not_same(config_a, config_a2)
  end

  def test_cache_with_used_constraints
    provider = Optify::OptionsProvider
               .build('../../tests/test_suites/conditions/configs')
               .init
    cache_options = Optify::CacheOptions.new
    preferences = Optify::GetOptionsPreferences.new
    config = provider.get_options('config', ['docs_example'], ExampleConfig, cache_options, preferences)
    assert_equal('from docs_example', config.key)
    assert_equal('only in docs_example', config.key_docs_example)
    config2 = provider.get_options('config', ['D'], ExampleConfig, cache_options, preferences)
    assert_same(config, config2)

    preferences.constraints = { clientId: 1234 }
    config2 = provider.get_options('config', ['docs_example'], ExampleConfig, cache_options, preferences)
    assert_equal(config, config2)
    # It could be the same one day, but right now we don't detect that the same features were used.
    assert_not_same(config, config2)
  end
end
