# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require 'liquid'

module Optify
  # Handles configurable string values with Liquid template support
  class ConfigurableString
    #: (String | Hash[String, String])
    attr_reader :base

    #: Hash[String, untyped]
    attr_reader :arguments

    #: Hash[String, String]
    attr_reader :loaded_files

    #: ((String | Hash[String, String]) base, Hash[String, untyped] arguments, Hash[String, String] loaded_files) -> void
    def initialize(base, arguments, loaded_files)
      @base = base
      @arguments = arguments
      @loaded_files = loaded_files
    end

    #: (Hash[String, untyped] data, Hash[String, String] loaded_files) -> ConfigurableString
    def self.from_hash(data, loaded_files)
      base = data['base'] || ''
      arguments = data['arguments'] || {}

      new(base, arguments, loaded_files)
    end

    #: -> String
    def resolve
      template_str = resolve_base
      return template_str if @arguments.empty?

      template = Liquid::Template.parse(template_str)
      template.render(@arguments)
    end

    #: -> String
    def resolve_base
      result = case @base
               when String
                 @base
               when Hash
                 if @base.key?('liquid')
                   @base['liquid']
                 elsif @base.key?('file')
                   file_path = @base['file']
                   return '' unless file_path

                   # Look up file from pre-loaded files
                   contents = @loaded_files[file_path]
                   if contents.nil?
                     warn "File '#{file_path}' not found in loaded files"
                     return ''
                   end
                   contents
                 else
                   ''
                 end
               end
      result || ''
    end

    #: (untyped data, Hash[String, String] loaded_files) -> untyped
    def self.process_value(data, loaded_files)
      case data
      when Hash
        if data.key?('$type') && data['$type'] == 'Optify.ConfigurableString'
          from_hash(data, loaded_files).resolve
        else
          data.transform_values { |v| process_value(v, loaded_files) }
        end
      when Array
        data.map { |v| process_value(v, loaded_files) }
      else
        data
      end
    end
  end
end
