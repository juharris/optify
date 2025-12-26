# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'

require_relative './base_config'
require_relative './cache_init_options'
require_relative './options_metadata'
require_relative './provider_module'

# Tools for working with configurations declared in files.
module Optify
  # Options for caching.
  # Only enabling or disabling caching is supported for now.
  class CacheOptions < FromHashable
  end

  # Provides configurations based on keys and enabled feature names.
  class OptionsProvider
    include ProviderModule

    # TODO: Find a better way to proxy the methods with copying the parameters.

    #: -> Hash[String, OptionsMetadata]
    def features_with_metadata
      _features_with_metadata
    end

    #: [Config] (String key, Array[String] feature_names, Class[Config] config_class, ?CacheOptions? cache_options, ?Optify::GetOptionsPreferences? preferences) -> Config
    def get_options(key, feature_names, config_class, cache_options = nil, preferences = nil)
      _get_options(key, feature_names, config_class, cache_options, preferences)
    end

    # (Optional) Eagerly initializes the cache.
    # @param cache_init_options Options for initializing the cache.
    # @return [OptionsProvider] `self`.
    #: (?CacheInitOptions?) -> OptionsProvider
    def init(cache_init_options = nil)
      _init(cache_init_options)
      self
    end
  end
end
