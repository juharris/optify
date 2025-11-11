# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require 'listen'
require_relative 'options_provider_impl'

module Optify
  # Watcher implementation that rebuilds provider on file changes
  class OptionsWatcherImpl < OptionsProviderImpl
    extend T::Sig

    #: Array[Float]?
    attr_reader :last_modified

    #: (
    #|   Hash[String, Feature] features,
    #|   Hash[String, String] aliases,
    #|   Array[String] config_directories,
    #|   ^-> OptionsProviderImpl builder
    #| ) -> void
    def initialize(features, aliases, config_directories, builder)
      super(features, aliases, config_directories)
      @builder = builder
      @listener = nil
      @last_modified = nil
      @mutex = Mutex.new
      update_last_modified
    end

    #: -> void
    def start_watching
      return if @listener

      @listener = Listen.to(*@config_directories) do |_modified, _added, _removed|
        rebuild_if_needed
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
      return unless has_changes?

      rebuild
    end

    #: -> bool
    def has_changes?
      current_modified = calculate_last_modified
      @last_modified != current_modified
    end

    #: -> void
    def rebuild
      @mutex.synchronize do
        puts '[optify] Rebuilding OptionsProvider because contents at path(s) changed: ' \
             "#{@config_directories.to_set}"

        provider = @builder.call
        @features = provider.features
        @aliases = provider.aliases
        @cache = nil
        @features_with_metadata = nil
        update_last_modified

        puts "\e[32m[optify] Successfully rebuilt the OptionsProvider.\e[0m"
      rescue StandardError => e
        puts "\e[31m[optify] Failed to rebuild OptionsProvider: #{e.message}\e[0m"
      end
    end

    private

    #: -> void
    def update_last_modified
      @last_modified = calculate_last_modified
    end

    #: -> Array[Float]
    def calculate_last_modified
      timestamps = []
      @config_directories.each do |dir|
        next unless Dir.exist?(dir)

        Dir.glob(File.join(dir, '**', '*.{json,json5,yaml,yml}')).each do |file|
          next if file.include?('/.optify/')

          timestamps << File.mtime(file).to_f
        end

        timestamps << File.mtime(dir).to_f if File.directory?(dir)
      end
      timestamps.sort
    end
  end
end
