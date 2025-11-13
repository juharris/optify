# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require 'listen'
require_relative 'options_provider_impl'

module Optify
  # Watcher implementation that rebuilds provider on file changes
  class OptionsWatcherImpl < OptionsProviderImpl
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
      return if @config_directories.empty?

      dirs = @config_directories
      @listener = case dirs.size
                  when 1
                    Listen.to(dirs.fetch(0), latency: 0.1, wait_for_delay: 0.1) do |modified, added, _removed|
                      handle_file_changes(modified, added)
                    end
                  when 2
                    Listen.to(dirs.fetch(0), dirs.fetch(1), latency: 0.1, wait_for_delay: 0.1) do |modified, added, _removed|
                      handle_file_changes(modified, added)
                    end
                  else
                    first_three = dirs.take(3)
                    d0 = first_three[0] || ''
                    d1 = first_three[1] || ''
                    d2 = first_three[2] || ''
                    Listen.to(d0, d1, d2, latency: 0.1, wait_for_delay: 0.1) do |modified, added, _removed|
                      handle_file_changes(modified, added)
                    end
                  end
      @listener.start
      # Give Listen a brief moment to initialize its monitoring threads
      sleep 0.01
    end

    #: (Array[String] modified, Array[String] added) -> void
    def handle_file_changes(modified, added)
      # Filter for config files and exclude .optify directory
      all_changed = modified + added
      changed_files = all_changed.select { |path| path =~ /\.(json|json5|yaml|yml)$/ }
                                 .reject { |path| path.include?('/.optify/') }
      rebuild_if_needed unless changed_files.empty?
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
