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

    # Compare this object with another object for equality.
    # @param other The object to compare.
    # @return [Boolean] true if the objects are equal; otherwise, false.
    sig { params(other: T.untyped).returns(T::Boolean) }
    def ==(other); end
  end

  # Options for caching.
  # Only enabling or disabling caching is supported for now.
  class CacheOptions < BaseConfig
  end

  # Information about a feature.
  class OptionsMetadata < BaseConfig
    sig { returns(T.nilable(T::Array[String])) }
    def aliases; end

    sig { returns(T.untyped) }
    def details; end

    sig { returns(String) }
    def name; end

    sig { returns(T.nilable(String)) }
    def owners; end

    sig { returns(T.nilable(String)) }
    def path; end
  end

  # Preferences when getting options.
  class GetOptionsPreferences
    # Set constraints for the current request to limit the features that can be enabled.
    sig { params(value: T.nilable(T::Hash[T.untyped, T.untyped])).void }
    def constraints=(value); end

    sig { params(value: T.nilable(String)).void }
    def constraints_json=(value); end

    sig { returns(T.nilable(String)) }
    def constraints_json; end

    # Indicates if overrides are set.
    sig { returns(T::Boolean) }
    def overrides?; end

    # Set overrides to apply after building the options based on the feature names.
    # Do not provide overrides when requesting cached options.
    # @param value The overrides to apply.
    sig { params(value: T.nilable(T::Hash[T.untyped, T.untyped])).void }
    def overrides=(value); end

    # Set overrides to apply after building the options based on the feature names.
    # Do not provide overrides when requesting cached options.
    # @param value The overrides to apply as serialized JSON.
    sig { params(value: T.nilable(String)).void }
    def overrides_json=(value); end

    sig { returns(T.nilable(String)) }
    def overrides_json; end

    sig { params(value: T::Boolean).void }
    def skip_feature_name_conversion=(value); end

    sig { returns(T::Boolean) }
    def skip_feature_name_conversion; end
  end

  # A registry of features that provides configurations.
  class OptionsRegistry
    include ProviderModule

    abstract!

    class << self
      # Build using just one directory.
      # @param directory The directory to build the provider from.
      # @return The instance.
      sig { params(directory: String).returns(T.attached_class) }
      def build(directory); end

      # Build using just one directory and enforce a schema for all feature files.
      # @param directory The directory to build the provider from.
      # @param schema The schema to enforce.
      # @return The instance.
      sig { params(directory: String, schema: String).returns(T.attached_class) }
      def build_with_schema(directory, schema); end

      # Build from multiple directories.
      # @param directories The directories to build the provider from.
      # @return The instance.
      sig { params(directories: T::Array[String]).returns(T.attached_class) }
      def build_from_directories(directories); end

      # Build from multiple directories and enforce a schema for all feature files.
      # @param directories The directories to build the provider from.
      # @param schema The schema to enforce.
      # @return The instance.
      sig { params(directories: T::Array[String], schema: String).returns(T.attached_class) }
      def build_from_directories_with_schema(directories, schema); end
    end

    # @return All of the aliases.
    sig { returns(T::Array[String]) }
    def aliases; end

    # @return All of the aliases and features.
    sig { returns(T::Array[String]) }
    def features_and_aliases; end

    # @return All of the canonical feature names.
    sig { returns(T::Array[String]) }
    def features; end

    # @return All of the keys and values for the the features.
    sig { returns(T::Hash[String, OptionsMetadata]) }
    def features_with_metadata; end

    # @return All of the keys and values for the the features.
    sig do
      params(feature_names: T::Array[String], preferences: GetOptionsPreferences)
        .returns(String)
    end
    def get_all_options_json(feature_names, preferences); end
  end

  # A module only for internal use that provides the methods to help implement providers.
  # Some of the methods shown within this module are implemented in Rust
  # and are declared in this common module to avoid duplicate declarations in different classes.
  module ProviderModule
    # Map an alias or canonical feature name (perhaps derived from a file name) to a canonical feature name.
    # Canonical feature names map to themselves.
    #
    # @param feature_name The name of an alias or a feature.
    # @return The canonical feature name.
    sig { params(feature_name: String).returns(String) }
    def get_canonical_feature_name(feature_name); end

    # Map aliases or canonical feature names (perhaps derived from a file names) to the canonical feature names.
    # Canonical feature names map to themselves.
    # This implementation may do an optimization for small arrays.
    #
    # @param feature_names The names of aliases or features.
    # @return The canonical feature names.
    sig { params(feature_names: T::Array[String]).returns(T::Array[String]) }
    def get_canonical_feature_names(feature_names); end

    # @return The metadata for the feature.
    sig { params(canonical_feature_name: String).returns(T.nilable(OptionsMetadata)) }
    def get_feature_metadata(canonical_feature_name); end

    # Fetches options based on the provided key and feature names.
    #
    # @param key The key to fetch options for.
    # @param feature_names The enabled feature names to use to build the options.
    # @param config_class The class of the configuration to return.
    # The class must implement `from_hash` as a class method to convert a hash to an instance of the class.
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
    # @return `self`.
    sig { returns(T.self_type) }
    def init; end

    private

    # Map aliases or canonical feature names (perhaps derived from a file names) to the canonical feature names.
    # Canonical feature names map to themselves.
    # This implementation calls the Rust implementation directly.
    #
    # @param feature_names The names of aliases or features.
    # @return The canonical feature names.
    sig { params(feature_names: T::Array[String]).returns(T::Array[String]) }
    def _get_canonical_feature_names(feature_names); end

    # @return The metadata for the feature.
    sig { params(canonical_feature_name: String).returns(T.nilable(String)) }
    def get_feature_metadata_json(canonical_feature_name); end

    # @return All of the keys and values for the the features.
    sig { returns(String) }
    def features_with_metadata_json; end
  end

  # Provides configurations based on keys and enabled feature names.
  class OptionsProvider < OptionsRegistry
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

  # Like `OptionsProvider` but also watches for changes to the files and reloads the options.
  class OptionsWatcher < OptionsRegistry
    # @return [Time] Returns the time when the provider was finished building.
    sig { returns(Time) }
    def last_modified; end
  end

  # A builder for creating an `OptionsWatcher` instance.
  #
  # This builder is kept separate from the `OptionsProviderBuilder`
  # in order to keep `OptionsProviderBuilder` and `OptionsProvider` as simple and efficient as possible for production use.
  class OptionsWatcherBuilder
    # Adds a directory to watch for changes.
    #
    # @param path [String] The path of the directory to add.
    # @return [OptionsWatcherBuilder] `self`.
    sig { params(path: String).returns(OptionsWatcherBuilder) }
    def add_directory(path); end

    # @return [OptionsWatcher] A newly built `OptionsWatcher`.
    sig { returns(OptionsWatcher) }
    def build; end
  end
end
