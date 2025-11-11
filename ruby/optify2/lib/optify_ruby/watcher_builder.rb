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
        aliases = {}

        @directories.each do |dir|
          load_directory(dir, features, aliases)
        end

        OptionsProviderImpl.new(features, aliases, @directories)
      end

      provider = builder_proc.call
      OptionsWatcherImpl.new(
        provider.features,
        provider.aliases,
        @directories,
        builder_proc
      )
    end
  end
end
