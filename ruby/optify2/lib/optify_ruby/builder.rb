# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require_relative 'feature'
require_relative 'options_provider_impl'
require_relative 'config_loader'

module Optify
  # Builder for constructing OptionsProvider instances
  class OptionsProviderBuilderImpl
    extend T::Sig

    #: -> void
    def initialize
      @directories = [] #: Array[String]
    end

    #: (String) -> OptionsProviderBuilderImpl
    def add_directory(directory)
      @directories << directory
      self
    end

    #: -> OptionsProviderImpl
    def build
      features = {}
      aliases = {}

      @directories.each do |dir|
        load_directory(dir, features, aliases)
      end

      OptionsProviderImpl.new(features, aliases, @directories)
    end

    private

    #: (String, Hash[String, Feature], Hash[String, String]) -> void
    def load_directory(directory, features, aliases)
      return unless Dir.exist?(directory)

      Dir.glob(File.join(directory, '**', '*.{json,json5,yaml,yml}')).each do |file_path|
        next if file_path.include?('/.optify/')

        feature_name = extract_feature_name(file_path, directory)
        next unless feature_name

        feature = Feature.from_file(file_path, feature_name)
        features[feature_name] = feature

        feature.metadata.aliases&.each do |alias_name|
          aliases[alias_name.downcase] = feature_name
        end
      end

      features.each_key do |fname|
        aliases[fname.downcase] = fname
      end
    end

    #: (String, String) -> String?
    def extract_feature_name(file_path, base_dir)
      relative_path = file_path.sub(base_dir, '').sub(%r{^/}, '')
      return nil if relative_path.start_with?('.optify')

      relative_path.sub(File.extname(relative_path), '').tr('/', '/')
    end
  end
end
