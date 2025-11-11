# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require_relative 'options_metadata'

module Optify
  # Represents a feature with its configuration options and metadata
  class Feature
    extend T::Sig

    #: String
    attr_reader :name

    #: String
    attr_reader :file_path

    #: Hash[String, untyped]?
    attr_reader :conditions

    #: Hash[String, Hash[String, untyped]]
    attr_reader :options

    #: OptionsMetadata
    attr_reader :metadata

    #: (
    #|   String name,
    #|   String file_path,
    #|   Hash[String, untyped]? conditions,
    #|   Hash[String, Hash[String, untyped]] options,
    #|   OptionsMetadata metadata
    #| ) -> void
    def initialize(name, file_path, conditions, options, metadata)
      @name = name
      @file_path = file_path
      @conditions = conditions
      @options = options
      @metadata = metadata
    end

    #: (String file_path, String name) -> Feature
    def self.from_file(file_path, name)
      require_relative 'config_loader'
      data = ConfigLoader.load_file(file_path)

      conditions = data['conditions']
      options = data['options'] || {}
      metadata_data = data['metadata'] || {}

      metadata_hash = {
        'name' => name,
        'path' => file_path,
        'aliases' => metadata_data['aliases'] || [],
        'dependents' => metadata_data['dependents'] || [],
        'details' => metadata_data['details'],
        'owners' => metadata_data['owners']
      }

      metadata = OptionsMetadata.from_hash(metadata_hash)

      new(name, file_path, conditions, options, metadata)
    end
  end
end
