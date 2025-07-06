# frozen_string_literal: true
# typed: true

require 'json'
require 'test/unit'
require_relative '../lib/optify'
require_relative 'my_config'

class ProviderCacheTest < Test::Unit::TestCase
  def test_cache_with_constraints
    provider = Optify::OptionsProvider
               .build('../../tests/test_suites/simple/configs')
               .init
    cache_options = Optify::CacheOptions.new
    preferences = Optify::GetOptionsPreferences.new
    preferences.constraints = {
      wtv: 3
    }

    config_a = provider.get_options('myConfig', ['A'], MyConfig, cache_options, preferences)
    config_a2 = provider.get_options('myConfig', ['a'], MyConfig, cache_options, preferences)
    assert_same(config_a, config_a2)

    preferences.constraints = {
      wtv: 4
    }
    config_a2 = provider.get_options('myConfig', ['A'], MyConfig, cache_options, preferences)
    assert_not_same(config_a, config_a2)
  end
end
