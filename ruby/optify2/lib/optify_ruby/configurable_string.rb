# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require 'liquid'

module Optify
  # Handles configurable string values with Liquid template support
  class ConfigurableString
    extend T::Sig

    #: (String | Hash[String, String])
    attr_reader :base

    #: Hash[String, untyped]
    attr_reader :arguments

    #: String?
    attr_reader :base_dir

    #: ((String | Hash[String, String]) base, Hash[String, untyped] arguments, ?String? base_dir) -> void
    def initialize(base, arguments, base_dir = nil)
      @base = base
      @arguments = arguments
      @base_dir = base_dir
    end

    #: (Hash[String, untyped] data, ?String? base_dir) -> ConfigurableString
    def self.from_hash(data, base_dir = nil)
      base = data['base'] || ''
      arguments = data['arguments'] || {}

      new(base, arguments, base_dir)
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

                   file_path = File.join(@base_dir, file_path) if @base_dir
                   File.read(file_path)
                 else
                   ''
                 end
               end
      result || ''
    end

    #: (untyped data, ?String? base_dir) -> untyped
    def self.process_value(data, base_dir = nil)
      case data
      when Hash
        if data.key?('$type') && data['$type'] == 'Optify.ConfigurableString'
          from_hash(data, base_dir).resolve
        else
          data.transform_values { |v| process_value(v, base_dir) }
        end
      when Array
        data.map { |v| process_value(v, base_dir) }
      else
        data
      end
    end
  end
end
