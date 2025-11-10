# frozen_string_literal: true
# typed: strict

require 'sorbet-runtime'

module Optify
  # @!visibility private
  module ProviderModule
    #: (Array[String] feature_names) -> Array[String]
    def get_canonical_feature_names(feature_names)
      # Try to optimize a typical case where there are just a few features.
      # Ideally in production, a single feature that imports many other features is used for the most common scenarios.
      # Benchmarks show that it is faster to use a loop than to call the Rust implementation which involves making a `Vec<String>` and returning a `Vec<String>`.
      if feature_names.length < 4
        feature_names.map { |feature_name| get_canonical_feature_name(feature_name) }
      else
        _get_canonical_feature_names(feature_names)
      end
    end

    #: (String canonical_feature_name) -> Optify::OptionsMetadata?
    def get_feature_metadata(canonical_feature_name)
      metadata_json = get_feature_metadata_json(canonical_feature_name)
      return nil if metadata_json.nil?

      OptionsMetadata.from_hash(JSON.parse(metadata_json))
    end

    private

    #: -> Hash[String, OptionsMetadata]
    def _features_with_metadata
      return @features_with_metadata if @features_with_metadata

      result = JSON.parse(features_with_metadata_json)
      result.each do |key, value|
        result[key] = OptionsMetadata.from_hash(value)
      end
      result.freeze

      @features_with_metadata = result
      result
    end

    # Fetches options based on the provided key and feature names.
    #
    # @param key The key to fetch options for.
    # @param feature_names The enabled feature names to use to build the options.
    # @param config_class The class of the configuration to return.
    # The class must implement `from_hash` as a class method to convert a hash to an instance of the class.
    # It is recommended to use a class that extends `Optify::FromHashable` because it implements `from_hash`.
    # @param cache_options Set this if caching is desired. Only very simple caching is supported for now.
    # @param preferences The preferences to use when getting options.
    # @return The options.
    #: [Config] (String key, Array[String] feature_names, Class[Config] config_class, ?CacheOptions? cache_options, ?Optify::GetOptionsPreferences? preferences) -> Config
    def _get_options(key, feature_names, config_class, cache_options = nil, preferences = nil)
      return get_options_with_cache(key, feature_names, config_class, cache_options, preferences) if cache_options

      unless config_class.respond_to?(:from_hash)
        Kernel.raise NotImplementedError,
                     "The provided config class must implement `from_hash` as a class method
              in order to be converted.
              Recommended: extend `Optify::FromHashable`."
      end

      options_json = if preferences
                       get_options_json_with_preferences(key, feature_names, preferences)
                     else
                       get_options_json(key, feature_names)
                     end
      hash = JSON.parse(options_json)
      config_class #: as untyped
        .from_hash(hash)
    end

    #: -> void
    def _init
      @cache = {} #: Hash[untyped, untyped]?
      @features_with_metadata = nil #: Hash[String, OptionsMetadata]?
    end

    NOT_FOUND_IN_CACHE_SENTINEL = Object.new

    #: [Config] (String key, Array[String] feature_names, Class[Config] config_class, Optify::CacheOptions _cache_options, ?Optify::GetOptionsPreferences? preferences) -> Config
    def get_options_with_cache(key, feature_names, config_class, _cache_options, preferences = nil)
      # Cache directly in Ruby instead of Rust because:
      # * Avoid any possible conversion overhead.
      # * Memory management: probably better to do it in Ruby for a Ruby app and avoid memory in Rust.
      if preferences&.overrides?
        Kernel.raise ArgumentError,
                     'Caching when overrides are given is not supported. Do not pass cache options when using overrides in preferences.'
      end

      init unless @cache

      if preferences.nil?
        feature_names = get_filtered_features(feature_names, GetOptionsPreferences.new)
      elsif !preferences.skip_feature_name_conversion || preferences.constraints_json
        feature_names = get_filtered_features(feature_names, preferences)
      end

      # Features are filtered, so we don't need the constraints in the cache key.
      are_configurable_strings_enabled = preferences&.are_configurable_strings_enabled? || false
      cache_key = [key, feature_names, are_configurable_strings_enabled, config_class]
      result = @cache #: as !nil
               .fetch(cache_key, NOT_FOUND_IN_CACHE_SENTINEL)
      return result unless result.equal?(NOT_FOUND_IN_CACHE_SENTINEL)

      # Handle a cache miss.

      # We can avoid converting the features names because they're already converted from filtering above, if that was desired.
      # We don't need the constraints because we filtered the features above.
      # We already know there are no overrides because we checked above.
      preferences = GetOptionsPreferences.new
      preferences.skip_feature_name_conversion = true
      preferences.enable_configurable_strings if are_configurable_strings_enabled

      result = get_options(key, feature_names, config_class, nil, preferences)

      @cache #: as !nil
        .[]= cache_key, result
    end
  end
end
