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

      # If no arguments, return the base template as-is
      # (even if it contains liquid syntax, we can't resolve it without arguments)
      return template_str if @arguments.empty?

      # Use DynamicArguments to lazily resolve arguments on-demand
      dynamic_arguments = DynamicArguments.new(@arguments, @loaded_files)

      template = Liquid::Template.parse(template_str)
      template.render(dynamic_arguments)
    end

    #: -> String
    def resolve_base
      case @base
      when String
        @base
      when Hash
        if @base.key?('liquid')
          # Base is a liquid template
          @base['liquid'] || ''
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

    # Drop class for lazy argument resolution in Liquid templates
    # Mimics the Rust DynamicArguments implementation
    class DynamicArguments < Liquid::Drop
      #: (Hash[String, untyped] arguments, Hash[String, String] loaded_files) -> void
      def initialize(arguments, loaded_files)
        super()
        @arguments = arguments
        @loaded_files = loaded_files
        @cache = {} #: Hash[String, String]
      end

      #: (String) -> String?
      def [](key)
        return @cache[key] if @cache.key?(key)

        value = resolve_value(key)
        @cache[key] = value if value
        value
      end

      #: (String) -> String?
      def before_method(key)
        # Liquid calls this when accessing a variable
        self[key]
      end

      private

      #: (String) -> String?
      def resolve_value(key)
        replacement = @arguments[key]
        return nil unless replacement

        case replacement
        when String
          replacement
        when Hash
          if replacement.key?('file')
            file_path = replacement['file']
            contents = @loaded_files[file_path]
            if contents.nil?
              warn "File '#{file_path}' not found for key '#{key}'"
              return nil
            end

            # If the file is a liquid template, render it with the same arguments context
            if file_path.end_with?('.liquid')
              render_liquid(contents)
            else
              contents
            end
          elsif replacement.key?('liquid')
            render_liquid(replacement['liquid'])
          end
        end
      end

      #: (String) -> String?
      def render_liquid(template_str)
        template = Liquid::Template.parse(template_str)
        template.render(self)
      rescue Liquid::Error => e
        warn "Liquid error: #{e.message}"
        nil
      end
    end
  end
end
