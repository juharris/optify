# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require_relative 'base_config'
require_relative 'options_metadata'
require_relative 'options_registry'
require_relative 'options_provider_impl'
require_relative 'builder'

module Optify
  # Options for caching
  class CacheOptions < FromHashable
  end

  # Provides configurations based on keys and enabled feature names
  class OptionsProvider < OptionsRegistry
    #: ((OptionsProviderImpl | OptionsWatcherImpl) impl) -> OptionsProvider
    def self.from_impl(impl)
      provider = allocate
      provider.instance_variable_set(:@impl, impl)
      provider.instance_variable_set(:@cache, nil)
      provider.instance_variable_set(:@features_with_metadata, nil)
      provider
    end

    #: (String directory) -> OptionsProvider
    def self.build(directory)
      OptionsProviderBuilder.new.add_directory(directory).build
    end

    #: (Array[String] directories) -> OptionsProvider
    def self.build_from_directories(directories)
      builder = OptionsProviderBuilder.new
      directories.each { |dir| builder.add_directory(dir) }
      builder.build
    end

    #: [Config]
    #| (String, Array[String], Class[Config], ?CacheOptions?, ?Optify::GetOptionsPreferences?)
    #| -> Config
    def get_options(key, feature_names, config_class, cache_options = nil, preferences = nil)
      _get_options(key, feature_names, config_class, cache_options, preferences)
    end
  end

  # Builder for OptionsProvider
  class OptionsProviderBuilder
    #: -> void
    def initialize
      @builder = OptionsProviderBuilderImpl.new #: OptionsProviderBuilderImpl
    end

    #: (String directory) -> OptionsProviderBuilder
    def add_directory(directory)
      @builder.add_directory(directory)
      self
    end

    #: -> OptionsProvider
    def build
      impl = @builder.build
      OptionsProvider.from_impl(impl)
    end
  end
end
