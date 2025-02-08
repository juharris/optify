# frozen_string_literal: true
# typed: strict

require 'json'
require 'ostruct'

require 'sorbet-runtime'

require_relative './base_config'

# Tools for working with configurations declared in files.
module Optify
  class CacheOptions < BaseConfig
  end

  # Provides configurations based on keys and enabled feature names.
  class OptionsProvider
    extend T::Sig

    # Fetches options in JSON format based on the provided key and feature names.
    #
    # @param key [String] the key to fetch options for.
    # @param feature_names [Array<String>] The enabled feature names to use to build the options.
    # @return [String] the options in JSON.
    sig { params(key: String, feature_names: T::Array[String]).returns(String) }
    def get_options_json(key, feature_names)
      # Implemented in Rust.
      raise NotImplementedError
    end

    # Fetches options based on the provided key and feature names.
    #
    # @param key [String] the key to fetch options for.
    # @param feature_names [Array<String>] The enabled feature names to use to build the options.
    # @param config_class [ConfigType] The class of the configuration to return.
    # It is recommended to use a class that extends `Optify::BaseConfig` because it implements `from_hash`.
    # @param cache_options TODO
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

      options_json = get_options_json(key, feature_names)
      h = JSON.parse(options_json, object_class: Hash)
      unless config_class.respond_to?(:from_hash)
        raise NotImplementedError,
              "The provided config class must implement `from_hash` as a class method
            in order to be converted."
      end

      T.unsafe(config_class).from_hash(h)
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
      # TODO: Handle aliases. It will be extra work because the mapping is only visible in Rust for now
      # and we don't want the cache in Rust because we won't to avoid any conversion overhead.
      @cache ||= T.let({}, T.nilable(T::Hash[T.untyped, T.untyped]))
      cache_key = [key, feature_names, config_class]
      result = @cache.fetch(cache_key, NOT_FOUND_IN_CACHE_SENTINEL)
      return result unless result.equal?(NOT_FOUND_IN_CACHE_SENTINEL)

      result = get_options(key, feature_names, config_class)

      @cache[cache_key] = result
    end
  end

  # A builder for creating an `OptionsProvider` instance.
  class OptionsProviderBuilder
    extend T::Sig

    # Adds a directory to the builder.
    #
    # @param path [String] The path of the directory to add.
    # @return [OptionsProviderBuilder] `self`.
    sig { params(path: String).returns(OptionsProviderBuilder) }
    def add_directory(path)
      # Implemented in Rust.
      raise NotImplementedError
    end

    # @return [OptionsProvider] A newly built `OptionsProvider`.
    sig { returns(OptionsProvider) }
    def build
      # Implemented in Rust.
      raise NotImplementedError
    end
  end
end
