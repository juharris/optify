# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require_relative 'base_config'
require_relative 'options_metadata'
require_relative 'options_registry'
require_relative 'options_provider_impl'
require_relative 'builder'
require_relative 'config_loader'

module Optify
  # Options for caching
  class CacheOptions < FromHashable
  end

  # Provides configurations based on keys and enabled feature names
  class OptionsProvider < OptionsRegistry
    #: ((OptionsProviderImpl | OptionsWatcherImpl) impl) -> OptionsProvider
    def self.from_impl(impl)
      # FIXME: We don't need the indirection.
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

    #: (String directory, String schema_path) -> OptionsProvider
    def self.build_with_schema(directory, schema_path)
      # FIXME: Do proper full schema validation with a library.
      # Basic schema validation - check for obviously invalid properties
      schema = JSON.parse(File.read(schema_path))
      allowed_properties = schema['properties']&.keys || []

      # Validate each file against the schema before building
      Dir.glob(File.join(directory, '**', '*.{json,json5,yaml,yml}')).each do |file_path|
        next if file_path.include?('/.optify/')

        data = ConfigLoader.load_file(file_path)

        # Simple validation: check if there are any properties not in the schema
        next unless allowed_properties.any? && data.is_a?(Hash)

        invalid_keys = data.keys - allowed_properties
        raise "Schema validation failed for '#{file_path}': properties #{invalid_keys.inspect} not allowed" unless invalid_keys.empty?
      end

      # If validation passed, build normally
      build(directory)
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
      # FIXME: We don't need the indirection.
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
