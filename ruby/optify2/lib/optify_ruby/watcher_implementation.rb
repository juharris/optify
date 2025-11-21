# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require_relative 'options_watcher_impl'
require_relative 'watcher_builder'
require_relative 'implementation'

module Optify
  # Auto-reloading options provider with file watching
  class OptionsWatcher < OptionsRegistry
    #: -> void
    def initialize # rubocop:disable Lint/MissingSuper
      raise 'Do not instantiate OptionsWatcher directly. Use OptionsWatcherBuilder instead.'
    end

    #: ((OptionsProviderImpl | OptionsWatcherImpl) impl) -> OptionsWatcher
    def self.from_impl(impl)
      # FIXME: We don't need the indirection.
      watcher = allocate
      watcher.instance_variable_set(:@impl, impl)
      watcher.instance_variable_set(:@cache, nil)
      watcher.instance_variable_set(:@features_with_metadata, nil)
      watcher.instance_variable_set(:@cache_creation_time, nil)
      watcher
    end

    #: (String directory) -> OptionsWatcher
    def self.build(directory)
      OptionsWatcherBuilder.new.add_directory(directory).build
    end

    #: (Array[String] directories) -> OptionsWatcher
    def self.build_from_directories(directories)
      builder = OptionsWatcherBuilder.new
      directories.each { |dir| builder.add_directory(dir) }
      builder.build
    end

    #: (String directory, String schema_path) -> OptionsWatcher
    def self.build_with_schema(directory, schema_path)
      # FIXME: Re-use the code from the OptionsProvider implementation which will do proper full schema validation.
      # We should not need to eagerly validate the schema here since the OptionsProvider implementation will do it when loading
      # and this class should hold an implementation of an `OptionsProvider`.
      # Basic schema validation - check for obviously invalid properties
      require_relative 'config_loader'

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

    #: -> Time
    def last_modified
      watcher_impl.last_modified
    end

    #: -> void
    def start_watching
      watcher_impl.start_watching
    end

    #: -> void
    def stop_watching
      watcher_impl.stop_watching
    end

    #: -> OptionsWatcher
    def init
      super
      @cache_creation_time = Time.now #: Time?
      self
    end

    #: -> Hash[String, OptionsMetadata]
    def features_with_metadata
      _check_cache
      super
    end

    #: [Config]
    #| (String, Array[String], Class[Config], ?CacheOptions?, ?Optify::GetOptionsPreferences?)
    #| -> Config
    def get_options(key, feature_names, config_class, cache_options = nil, preferences = nil)
      _check_cache if cache_options
      _get_options(key, feature_names, config_class, cache_options, preferences)
    end

    private

    #: -> OptionsWatcherImpl
    def watcher_impl
      impl_val = impl
      raise 'Expected OptionsWatcherImpl' unless impl_val.is_a?(OptionsWatcherImpl)

      impl_val
    end

    #: -> void
    def _check_cache
      cache_time = @cache_creation_time
      return if cache_time && cache_time > watcher_impl.last_modified

      init
    end
  end

  # Builder for OptionsWatcher
  class OptionsWatcherBuilder
    #: -> void
    def initialize
      # FIXME: We don't need the indirection.
      @builder = OptionsWatcherBuilderImpl.new #: OptionsWatcherBuilderImpl
    end

    #: (String directory) -> OptionsWatcherBuilder
    def add_directory(directory)
      @builder.add_directory(directory)
      self
    end

    #: -> OptionsWatcher
    def build
      impl = @builder.build
      OptionsWatcher.from_impl(impl)
    end
  end
end
