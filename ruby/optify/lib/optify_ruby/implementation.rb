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

  # Provides configurations based on keys and enabled feature names.
  class OptionsProvider
    extend T::Sig

    # Fetches options based on the provided key and feature names.
    #
    # @param key [String] the key to fetch options for.
    # @param feature_names [Array<String>] The enabled feature names to use to build the options.
    # @param config_class [ConfigType] The class of the configuration to return.
    # It is recommended to use a class that extends `Optify::BaseConfig` because it implements `from_hash`.
    # @param cache_options Set this if caching is desired. Only very simple caching is supported for now.
    # @return [ConfigType] The options.
    sig do
      type_parameters(:Config)
        .params(
          key: String,
          feature_names: T::Array[String],
          config_class: T::Class[T.type_parameter(:Config)],
          cache_options: T.nilable(CacheOptions)
        )
        .returns(T.type_parameter(:Config))
    end
    def get_options(key, feature_names, config_class, cache_options = nil)
      return get_options_with_cache(key, feature_names, config_class, cache_options) if cache_options

      unless config_class.respond_to?(:from_hash)
        raise NotImplementedError,
              "The provided config class must implement `from_hash` as a class method
              in order to be converted.
              Recommended: extend `Optify::BaseConfig`."
      end

      options_json = get_options_json(key, feature_names)
      hash = JSON.parse(options_json)
      T.unsafe(config_class).from_hash(hash)
    end

    private

    NOT_FOUND_IN_CACHE_SENTINEL = Object.new

    sig do
      type_parameters(:Config)
        .params(
          key: String,
          feature_names: T::Array[String],
          config_class: T::Class[T.type_parameter(:Config)],
          _cache_options: CacheOptions
        )
        .returns(T.type_parameter(:Config))
    end
    def get_options_with_cache(key, feature_names, config_class, _cache_options)
      # Cache directly in Ruby instead of Rust because:
      # * Avoid any possible conversion overhead.
      # * Memory management: probably better to do it in Ruby for a Ruby app and avoid memory in Rust.
      # TODO: Consider aliases when caching. Right now, they are only visible in Rust
      # and we don't want the cache in Rust because we won't to avoid any conversion overhead.
      @cache ||= T.let({}, T.nilable(T::Hash[T.untyped, T.untyped]))
      cache_key = [key, feature_names, config_class]
      result = @cache.fetch(cache_key, NOT_FOUND_IN_CACHE_SENTINEL)
      return result unless result.equal?(NOT_FOUND_IN_CACHE_SENTINEL)

      result = get_options(key, feature_names, config_class)
      T.unsafe(result).freeze if T.unsafe(result).respond_to?(:freeze)

      @cache[cache_key] = result
    end
  end
end
