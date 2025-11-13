# typed: strict
# frozen_string_literal: true

require 'json'
require 'yaml'
require 'rb_json5'
require 'sorbet-runtime'

module Optify
  # Loads configuration files (JSON, YAML, JSON5)
  module ConfigLoader
    extend T::Sig

    #: (String file_path) -> Hash[String, untyped]
    def self.load_file(file_path)
      content = File.read(file_path)
      ext = File.extname(file_path).downcase

      case ext
      when '.json', '.json5'
        load_json_with_comments(content)
      when '.yaml', '.yml'
        load_yaml(content)
      else
        raise ArgumentError, "Unsupported file type: #{ext}"
      end
    end

    #: (String content) -> Hash[String, untyped]
    def self.load_json_with_comments(content)
      RbJSON5.parse(content)
    end

    #: (String content) -> Hash[String, untyped]
    def self.load_yaml(content)
      # FIXME: Can we remove other args?
      result = YAML.safe_load(content, permitted_classes: [], permitted_symbols: [])
      # FIXME: Assert it is a hash or assume it is.
      result.is_a?(Hash) ? result : {}
    end
  end
end
