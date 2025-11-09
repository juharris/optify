# frozen_string_literal: true
# typed: strict

require 'sorbet-runtime'

require_relative './base_config'
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
    # @return [OptionsProvider] `self`.
    #: -> OptionsProvider
    def init
      _init
      self
    end
  end
end
