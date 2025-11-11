# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require 'json'

module Optify
  # Options for handling files in a directory
  class BuilderOptions
    extend T::Sig

    #: bool
    attr_reader :are_configurable_strings_enabled

    #: (bool are_configurable_strings_enabled) -> void
    def initialize(are_configurable_strings_enabled: false)
      @are_configurable_strings_enabled = are_configurable_strings_enabled
    end

    #: -> BuilderOptions
    def self.default
      new
    end

    #: (String file_path) -> BuilderOptions
    def self.from_file(file_path)
      data = JSON.parse(File.read(file_path))
      new(
        are_configurable_strings_enabled: data['areConfigurableStringsEnabled'] || false
      )
    end
  end
end
