# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require_relative 'options_watcher_impl'
require_relative 'watcher_builder'
require_relative 'implementation'

module Optify
  # Auto-reloading options provider with file watching
  class OptionsWatcher < OptionsProvider
    #: -> void
    def initialize
      raise 'Do not instantiate OptionsWatcher directly. Use OptionsWatcherBuilder instead.'
    end

    # @override
    #: ((OptionsProviderImpl | OptionsWatcherImpl) impl) -> OptionsProvider
    def self.from_impl(impl)
      watcher = allocate
      watcher.instance_variable_set(:@impl, impl)
      watcher.instance_variable_set(:@cache, nil)
      watcher.instance_variable_set(:@features_with_metadata, nil)
      watcher.instance_variable_set(:@cache_creation_time, nil)
      watcher
    end

    #: -> Array[Float]?
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

    #: -> void
    def rebuild_if_needed
      watcher_impl.rebuild_if_needed
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
      return if cache_time && watcher_impl.last_modified && cache_time.to_f > watcher_impl.last_modified.max

      init
    end
  end

  # Builder for OptionsWatcher
  class OptionsWatcherBuilder
    #: -> void
    def initialize
      @builder = OptionsWatcherBuilderImpl.new
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
