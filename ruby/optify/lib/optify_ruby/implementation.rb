# frozen_string_literal: true
# typed: strict

require 'json'
require 'ostruct'

require 'sorbet-runtime'

# Tools for working with configurations declared in files.
module Optify
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
    # @return [OpenStruct] the options.
    sig { params(key: String, feature_names: T::Array[String]).returns(OpenStruct) }
    def get_options(key, feature_names)
      options_json = get_options_json(key, feature_names)
      JSON.parse(options_json, object_class: OpenStruct)
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