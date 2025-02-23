# frozen_string_literal: true
# typed: strong

# Tools for working with configurations declared in files.
module Optify
  # A base class for classes from configuration files.
  # Classes that derive from this can easily be used with `Optify::OptionsProvider.get_options`
  # because they will have an implementation of `from_hash` that works recursively.
  # This class is a work in progress with minimal error handling
  # and doesn't handle certain cases such as nilable types yet.
  # It may be moved to another gem in the future.
  class BaseConfig
    abstract!

    # Create a new instance of the class from a hash.
    #
    # This is a class method that so that it can set members with private setters.
    # @param hash The hash to create the instance from.
    # @return The new instance.
    sig { params(hash: T::Hash[T.untyped, T.untyped]).returns(T.attached_class) }
    def self.from_hash(hash); end
  end

  # Options for caching.
  # Only enabling or disabling caching is supported for now.
  class CacheOptions < BaseConfig
  end

  # Preferences when getting options.
  class GetOptionsPreferences
    sig { params(value: T::Boolean).returns(GetOptionsPreferences) }
    def skip_feature_name_conversion=(value); end
    sig { returns(T::Boolean) }
    def skip_feature_name_conversion; end
  end

  # Provides configurations based on keys and enabled feature names.
  class OptionsProvider
    # Map an alias or canonical feature name (perhaps derived from a file name) to a canonical feature name.
    # Canonical feature names map to themselves.
    #
    # @param feature_name The name of an alias or a feature.
    # @return The canonical feature name.
    sig { params(feature_name: String).returns(String) }
    def get_canonical_feature_name(feature_name); end

    # Fetches options based on the provided key and feature names.
    #
    # @param key The key to fetch options for.
    # @param feature_names The enabled feature names to use to build the options.
    # @param config_class The class of the configuration to return.
    # It is recommended to use a class that extends `Optify::BaseConfig` because it implements `from_hash`.
    # @param cache_options Set this if caching is desired. Only very simple caching is supported for now.
    # @param preferences The preferences to use when getting options.
    # @return The options.
    sig do
      type_parameters(:Config)
        .params(
          key: String,
          feature_names: T::Array[String],
          config_class: T::Class[T.type_parameter(:Config)],
          cache_options: T.nilable(CacheOptions),
          preferences: T.nilable(Optify::GetOptionsPreferences)
        )
        .returns(T.type_parameter(:Config))
    end
    def get_options(key, feature_names, config_class, cache_options = nil, preferences = nil); end

    # Fetches options in JSON format based on the provided key and feature names.
    #
    # @param key [String] the key to fetch options for.
    # @param feature_names [Array<String>] The enabled feature names to use to build the options.
    # @return [String] the options in JSON.
    sig { params(key: String, feature_names: T::Array[String]).returns(String) }
    def get_options_json(key, feature_names); end

    # Fetches options in JSON format based on the provided key and feature names.
    #
    # @param key [String] the key to fetch options for.
    # @param feature_names [Array<String>] The enabled feature names to use to build the options.
    # @param preferences [GetOptionsPreferences] The preferences to use when getting options.
    # @return [String] the options in JSON.
    sig do
      params(key: String, feature_names: T::Array[String], preferences: GetOptionsPreferences)
        .returns(String)
    end
    def get_options_json_with_preferences(key, feature_names, preferences); end

    # (Optional) Eagerly initializes the cache.
    # @return [OptionsProvider] `self`.
    sig { returns(OptionsProvider) }
    def init; end
  end

  # A builder for creating an `OptionsProvider` instance.
  class OptionsProviderBuilder
    # Adds a directory to the builder.
    #
    # @param path [String] The path of the directory to add.
    # @return [OptionsProviderBuilder] `self`.
    sig { params(path: String).returns(OptionsProviderBuilder) }
    def add_directory(path); end

    # @return [OptionsProvider] A newly built `OptionsProvider`.
    sig { returns(OptionsProvider) }
    def build; end
  end
end
