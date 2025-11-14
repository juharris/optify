# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require 'listen'
require_relative 'options_provider_impl'

module Optify
  # Watcher implementation that rebuilds provider on file changes
  class OptionsWatcherImpl < OptionsProviderImpl
    # Let us send a splat to `Listen.to`.
    ListenClass = Listen #: untyped

    #: Time
    attr_reader :last_modified

    #: (
    #|   Hash[String, Feature] features,
    #|   Hash[String, String] alias_map,
    #|   Array[String] config_directories,
    #|   bool are_configurable_strings_enabled,
    #|   ^-> OptionsProviderImpl builder
    #| ) -> void
    def initialize(features, alias_map, config_directories, are_configurable_strings_enabled, builder)
      super(features, alias_map, config_directories, are_configurable_strings_enabled)
      @builder = builder #: ^-> OptionsProviderImpl
      @listener = nil #: Listen::Listener?
      @last_modified = Time.now #: Time
      @mutex = Mutex.new #: Mutex
    end

    #: -> void
    def start_watching
      return if @listener
      return if @config_directories.empty?

      dirs = @config_directories
      latency = 1
      wait_for_delay = 1
      @listener = ListenClass.to(*dirs, latency: latency, wait_for_delay: wait_for_delay) do |_modified, _added, _removed|
        rebuild
      end
      @listener.start
      # Give Listen a brief moment to initialize its monitoring threads
      sleep 0.01
    end

    #: -> void
    def stop_watching
      @listener&.stop
      @listener = nil
    end

    #: -> void
    def rebuild
      @mutex.synchronize do
        puts '[optify] Rebuilding OptionsProvider because contents at path(s) changed: ' \
             "#{@config_directories.to_set}"

        provider = @builder.call
        @features = provider.features
        @alias_map = provider.alias_map
        @cache = nil
        @features_with_metadata = nil
        @last_modified = Time.now

        puts "\e[32m[optify] Successfully rebuilt the OptionsProvider.\e[0m"
      rescue StandardError => e
        puts "\e[31m[optify] Failed to rebuild OptionsProvider: #{e.message}\e[0m"
      end
    end
  end
end
