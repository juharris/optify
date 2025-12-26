# typed: true
# frozen_string_literal: true

require 'test/unit'
require 'optify'
require_relative 'my_config'

class CacheModesTest < Test::Unit::TestCase
  PROVIDERS = [
    Optify::OptionsProvider,
    Optify::OptionsWatcher
  ].freeze
  CACHE_MODES = [
    Optify::CacheMode::NOT_THREAD_SAFE,
    Optify::CacheMode::THREAD_SAFE
  ].freeze

  def test_cache_respects_max_size
    PROVIDERS.each do |klass|
      CACHE_MODES.each do |mode|
        cache_init_options = Optify::CacheInitOptions.new(
          max_size: 2,
          mode: mode
        )
        provider = klass.build('../../tests/test_suites/simple/configs')
                        .init(cache_init_options)
        cache_options = Optify::CacheOptions.new

        # Cache first two configs
        config_a = provider.get_options('myConfig', ['A'], MyConfig, cache_options)
        config_b = provider.get_options('myConfig', ['B'], MyConfig, cache_options)

        # Verify they are cached
        config_a2 = provider.get_options('myConfig', ['A'], MyConfig, cache_options)
        config_b2 = provider.get_options('myConfig', ['B'], MyConfig, cache_options)
        assert_same(config_a, config_a2, 'config_a should be cached')
        assert_same(config_b, config_b2, 'config_b should be cached')

        # Add a third config, which should evict the LRU entry (config_a)
        config_ab = provider.get_options('myConfig', %w[A B], MyConfig, cache_options)
        config_ab2 = provider.get_options('myConfig', %w[A B], MyConfig, cache_options)
        assert_same(config_ab, config_ab2, 'config_ab should be cached')

        # config_a should have been evicted (LRU), config_b should still be cached
        config_b3 = provider.get_options('myConfig', ['B'], MyConfig, cache_options)
        assert_same(config_b, config_b3, 'config_b should still be cached')

        config_a3 = provider.get_options('myConfig', ['A'], MyConfig, cache_options)
        assert_not_same(config_a, config_a3, 'config_a should have been evicted and recreated')
      end
    end
  end
end
