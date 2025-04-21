# frozen_string_literal: true
# typed: strict

require 'json'

require 'sorbet-runtime'

require_relative './base_config'
require_relative './implementation'
require_relative './options_metadata'
require_relative './provider_module'

module Optify
  class OptionsWatcher # rubocop:disable Style/Documentation
    include ProviderModule

    #: -> Hash[String, OptionsMetadata]
    def features_with_metadata
      _check_cache
      _features_with_metadata
    end

    # Fetches options based on the provided key and feature names.
    #
    # @param key The key to fetch options for.
    # @param feature_names The enabled feature names to use to build the options.
    # @param config_class The class of the configuration to return.
    # It is recommended to use a class that extends `Optify::BaseConfig` because it implements `from_hash`.
    # @param cache_options Set this if caching is desired. Only very simple caching is supported for now.
    # @param preferences The preferences to use when getting options.
    # @return The options.
    #: [Config] (String key, Array[String] feature_names, Class[Config] config_class, ?CacheOptions? cache_options, ?Optify::GetOptionsPreferences? preferences) -> Config
    def get_options(key, feature_names, config_class, cache_options = nil, preferences = nil)
      if cache_options
        _check_cache
        return get_options_with_cache(key, feature_names, config_class, cache_options, preferences)
      end

      unless config_class.respond_to?(:from_hash)
        raise NotImplementedError,
              "The provided config class must implement `from_hash` as a class method
              in order to be converted.
              Recommended: extend `Optify::BaseConfig`."
      end

      options_json = if preferences
                       get_options_json_with_preferences(key, feature_names, preferences)
                     else
                       get_options_json(key, feature_names)
                     end
      hash = JSON.parse(options_json)
      T.unsafe(config_class).from_hash(hash)
    end

    # (Optional) Eagerly initializes the cache.
    # @return [OptionsWatcher] `self`.
    #: -> OptionsWatcher
    def init
      _init
      @cache_creation_time = T.let(Time.now, T.nilable(Time))
      self
    end

    private

    #: -> void
    def _check_cache
      return unless !@cache_creation_time || @cache_creation_time < last_modified

      # The cache is not setup or it is out of date.
      init
    end
  end
end
