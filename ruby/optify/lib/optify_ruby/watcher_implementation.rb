# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'

require_relative './base_config'
require_relative './implementation'
require_relative './options_metadata'
require_relative './provider_module'

module Optify
  # @!visibility private
  class OptionsWatcher
    include ProviderModule

    # TODO: Find a better way to proxy the methods with copying the parameters.

    #: -> Hash[String, OptionsMetadata]
    def features_with_metadata
      _check_cache
      _features_with_metadata
    end

    #: [Config] (String key, Array[String] feature_names, Class[Config] config_class, ?CacheOptions? cache_options, ?Optify::GetOptionsPreferences? preferences) -> Config
    def get_options(key, feature_names, config_class, cache_options = nil, preferences = nil)
      _check_cache if cache_options

      _get_options(key, feature_names, config_class, cache_options, preferences)
    end

    # (Optional) Eagerly initializes the cache.
    # @return [OptionsWatcher] `self`.
    #: -> OptionsWatcher
    def init
      _init
      @cache_creation_time = Time.now #: Time?
      self
    end

    private

    #: -> void
    def _check_cache
      return if @cache_creation_time && @cache_creation_time > last_modified

      # The cache is not setup or it is out of date.
      init
    end
  end
end
