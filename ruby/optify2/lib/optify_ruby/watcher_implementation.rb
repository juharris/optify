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
