# frozen_string_literal: true
# typed: strict

require 'json'

require 'sorbet-runtime'

require_relative './base_config'

# Tools for working with configurations declared in files.
module Optify
  # Options for caching.
  # Only enabling or disabling caching is supported for now.
  class CacheOptions < BaseConfig
  end

  # Information about a feature.
  class OptionsMetadata < BaseConfig
    sig { returns(T.nilable(T::Array[String])) }
    attr_reader :aliases

    sig { returns(T.untyped) }
    attr_reader :details

    sig { returns(String) }
    attr_reader :name

    sig { returns(T.nilable(String)) }
    attr_reader :owners

    private

    # To please Sorbet.
    sig { params(name: String).void }
    def initialize(name)
      super()
      @aliases = T.let(nil, T.nilable(T::Array[String]))
      @details = T.let(nil, T.untyped)
      @name = T.let(name, String)
      @owners = T.let(nil, T.nilable(String))
    end
  end

  # Provides configurations based on keys and enabled feature names.
  class OptionsProvider
    extend T::Sig

    sig { returns(T::Hash[String, OptionsMetadata]) }
    def features_with_metadata
      return @features_with_metadata if @features_with_metadata

      result = JSON.parse(features_with_metadata_json)
      result.each do |key, value|
        result[key] = OptionsMetadata.from_hash(value)
      end
      result.freeze

      @features_with_metadata = T.let(result, T.nilable(T::Hash[String, OptionsMetadata]))
      result
    end

    sig { params(canonical_feature_name: String).returns(T.nilable(Optify::OptionsMetadata)) }
    def get_feature_metadata(canonical_feature_name)
      features_with_metadata[canonical_feature_name]
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
    # @return [OptionsProvider] `self`.
    #: -> OptionsProvider
    def init
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
      feature_names = feature_names.map do |feature_name|
        get_canonical_feature_name(feature_name)
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
