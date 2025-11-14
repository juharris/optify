# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require_relative 'options_watcher_impl'
require_relative 'builder'
require_relative 'builder_options'

module Optify
  # Builder for constructing OptionsWatcher instances
  class OptionsWatcherBuilderImpl < OptionsProviderBuilderImpl
    #: -> OptionsWatcherImpl
    def build
      builder_proc = lambda do
        features = {}
        alias_map = {}
        are_configurable_strings_enabled = false #: bool

        @directories.each do |dir|
          load_directory(dir, features, alias_map)
        end

        # Use builder options from first directory (matching Rust behavior)
        first_dir = @directories.first
        if first_dir
          config_path = File.join(first_dir, '.optify', 'config.json')
          if File.exist?(config_path)
            builder_options = BuilderOptions.from_file(config_path)
            are_configurable_strings_enabled = builder_options.are_configurable_strings_enabled
          end
        end

        # Resolve imports for all features
        resolve_all_imports(features)

        OptionsProviderImpl.new(features, alias_map, @directories, are_configurable_strings_enabled)
      end

      provider = builder_proc.call
      watcher = OptionsWatcherImpl.new(
        provider.features,
        provider.alias_map,
        @directories,
        provider.instance_variable_get(:@are_configurable_strings_enabled),
        builder_proc
      )
      watcher.start_watching
      watcher
    end
  end
end
