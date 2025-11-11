# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require_relative 'options_watcher_impl'
require_relative 'builder'

module Optify
  # Builder for constructing OptionsWatcher instances
  class OptionsWatcherBuilderImpl < OptionsProviderBuilderImpl
    extend T::Sig

    #: -> OptionsWatcherImpl
    def build
      builder_proc = lambda do
        features = {}
        alias_map = {}

        @directories.each do |dir|
          load_directory(dir, features, alias_map)
        end

        # Resolve imports for all features
        resolve_all_imports(features)

        OptionsProviderImpl.new(features, alias_map, @directories, @builder_options)
      end

      provider = builder_proc.call
      OptionsWatcherImpl.new(
        provider.features,
        provider.alias_map,
        @directories,
        @builder_options,
        builder_proc
      )
    end
  end
end
