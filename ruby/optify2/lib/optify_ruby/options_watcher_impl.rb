# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require 'listen'
require_relative 'options_provider_impl'

module Optify
  # Watcher implementation that rebuilds provider on file changes
  class OptionsWatcherImpl < OptionsProviderImpl
    extend T::Sig

    #: Time
    attr_reader :last_modified

    #: (
    #|   Hash[String, Feature] features,
    #|   Hash[String, String] alias_map,
    #|   Array[String] config_directories,
    #|   BuilderOptions builder_options,
    #|   ^-> OptionsProviderImpl builder
    #| ) -> void
    def initialize(features, alias_map, config_directories, builder_options, builder)
      super(features, alias_map, config_directories, builder_options)
      @builder = builder #: ^-> OptionsProviderImpl
      @listener = nil #: Listen::Listener?
      @last_modified = Time.now #: Time
      @mutex = Mutex.new #: Mutex
    end

    #: -> void
    def start_watching
      return if @listener

      @listener = Listen.to(*@config_directories, latency: 0.1, wait_for_delay: 0.1) do |modified, added, _removed|
        # Filter for config files and exclude .optify directory
        all_changed = modified + added
        changed_files = all_changed.select { |path| path =~ /\.(json|json5|yaml|yml)$/ }
                                   .reject { |path| path.include?('/.optify/') }
        rebuild_if_needed unless changed_files.empty?
      end
      @listener&.start
    end

    #: -> void
    def stop_watching
      @listener&.stop
      @listener = nil
    end

    #: -> void
    def rebuild_if_needed
      rebuild
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
