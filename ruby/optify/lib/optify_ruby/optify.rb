# frozen_string_literal: true
# typed: strict

# Tools for working with configurations declared in files.
module Optify
  # Provides configurations based on keys and enabled feature names.
  class OptionsProvider
    extend T::Sig

    sig {params(provider: OptifyBindings::OptionsProvider).void}
    def initialize(provider)
      @provider = provider
    end

    # Fetches options in JSON format based on the provided key and feature names.
    #
    # @param key [String] the key to fetch options for
    # @param feature_names [Array<String>] the feature names to include in the options
    # @return [String] the options in JSON format
    sig { params(key: String, feature_names: T::Array[String]).returns(String) }
    def get_options_json(key, feature_names)
      @provider.get_options_json(key, feature_names)
    end
  end

  # This class provides a builder for creating an `OptionsProvider` instance.
  class OptionsProviderBuilder
    extend T::Sig

    sig { void }
    def initialize
      @builder = T.let(OptifyBindings::OptionsProviderBuilder.new, OptifyBindings::OptionsProviderBuilder)
    end

    # Adds a directory to the builder.
    #
    # @param path [String] the path of the directory to add
    # @return [OptionsProviderBuilder] the builder instance
    sig { params(path: String).returns(OptionsProviderBuilder) }
    def add_directory(path)
      @builder.add_directory(path)
      self
    end

    # Builds and returns an OptionsProvider instance.
    #
    # @return [OptionsProvider] the built `OptionsProvider` instance
    sig { returns(OptionsProvider) }
    def build
      OptionsProvider.new(@builder.build)
    end
  end
end