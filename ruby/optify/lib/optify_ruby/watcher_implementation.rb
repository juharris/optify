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
      result = JSON.parse(features_with_metadata_json)
      result.each do |key, value|
        result[key] = OptionsMetadata.from_hash(value)
      end
      result
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
      # FIXME: Maybe get rid of the cache because it will not get invalidated.
      return get_options_with_cache(key, feature_names, config_class, cache_options, preferences) if cache_options

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
      # FIXME: Maybe get rid of the cache because it will not get invalidated.
      @cache = T.let({}, T.nilable(T::Hash[T.untyped, T.untyped]))
      self
    end

    private

    NOT_FOUND_IN_CACHE_SENTINEL = Object.new

    #: [Config] (String key, Array[String] feature_names, Class[Config] config_class, Optify::CacheOptions _cache_options, ?Optify::GetOptionsPreferences? preferences) -> Config
    def get_options_with_cache(key, feature_names, config_class, _cache_options, preferences = nil)
      # Cache directly in Ruby instead of Rust because:
      # * Avoid any possible conversion overhead.
      # * Memory management: probably better to do it in Ruby for a Ruby app and avoid memory in Rust.
      init unless @cache
      unless preferences&.skip_feature_name_conversion
        # When there are just a few names, then it can be faster to convert them one by one in a loop to avoid working with an array in Rust.
        # When there are over 7 names, then it is faster to convert them with one call to Rust.
        feature_names = get_canonical_feature_names(feature_names)
      end

      cache_key = [key, feature_names, config_class]
      result = @cache&.fetch(cache_key, NOT_FOUND_IN_CACHE_SENTINEL)
      return result unless result.equal?(NOT_FOUND_IN_CACHE_SENTINEL)

      preferences ||= GetOptionsPreferences.new
      preferences.skip_feature_name_conversion = true
      result = get_options(key, feature_names, config_class, nil, preferences)

      T.must(@cache)[cache_key] = result
    end
  end
end
