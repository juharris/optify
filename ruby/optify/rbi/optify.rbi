# frozen_string_literal: true
# typed: strict

# Tools for working with configurations declared in files.
module Optify
  # Provides configurations based on keys and enabled feature names.
  class OptionsProvider
    # Fetches options based on the provided key and feature names.
    #
    # @param key [String] the key to fetch options for.
    # @param feature_names [Array<String>] The enabled feature names to use to build the options.
    # @param config_class [ConfigType] The class of the configuration to return.
    # @return [ConfigType] The options.
    sig do
      type_parameters(:Config)
        .params(
          key: String,
          feature_names: T::Array[String],
          config_class: T::Class[T.type_parameter(:Config)]
        )
        .returns(T.type_parameter(:Config))
    end
    def get_options(key, feature_names, config_class); end

    # Fetches options in JSON format based on the provided key and feature names.
    #
    # @param key [String] the key to fetch options for.
    # @param feature_names [Array<String>] The enabled feature names to use to build the options.
    # @return [String] the options in JSON.
    sig { params(key: String, feature_names: T::Array[String]).returns(String) }
    def get_options_json(key, feature_names); end
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
