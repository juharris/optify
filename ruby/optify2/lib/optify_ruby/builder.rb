# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require_relative 'feature'
require_relative 'options_provider_impl'
require_relative 'config_loader'
require_relative 'builder_options'

module Optify
  # Builder for constructing OptionsProvider instances
  class OptionsProviderBuilderImpl
    extend T::Sig

    #: -> void
    def initialize
      @directories = [] #: Array[String]
      @builder_options = BuilderOptions.default #: BuilderOptions
    end

    #: (String) -> OptionsProviderBuilderImpl
    def add_directory(directory)
      @directories << directory

      # Load .optify/config.json if it exists (use first directory's config)
      if @directories.length == 1
        config_path = File.join(directory, '.optify', 'config.json')
        @builder_options = File.exist?(config_path) ? BuilderOptions.from_file(config_path) : BuilderOptions.default
      end

      self
    end

    #: -> OptionsProviderImpl
    def build
      features = {}
      alias_map = {}

      @directories.each do |dir|
        load_directory(dir, features, alias_map)
      end

      # Resolve imports for all features
      resolve_all_imports(features)

      OptionsProviderImpl.new(features, alias_map, @directories, @builder_options)
    end

    private

    #: (String, Hash[String, Feature], Hash[String, String]) -> void
    def load_directory(directory, features, alias_map)
      return unless Dir.exist?(directory)

      Dir.glob(File.join(directory, '**', '*.{json,json5,yaml,yml}')).each do |file_path|
        next if file_path.include?('/.optify/')

        feature_name = extract_feature_name(file_path, directory)
        next unless feature_name

        feature = Feature.from_file(file_path, feature_name)
        features[feature_name] = feature

        # Add actual aliases to the map
        feature.metadata.aliases&.each do |alias_name|
          alias_map[alias_name.downcase] = feature_name
        end
      end

      # Add feature names themselves for case-insensitive lookup
      features.each_key do |fname|
        alias_map[fname.downcase] = fname
      end
    end

    #: (String, String) -> String?
    def extract_feature_name(file_path, base_dir)
      relative_path = file_path.sub(base_dir, '').sub(%r{^/}, '')
      return nil if relative_path.start_with?('.optify')

      relative_path.sub(File.extname(relative_path), '').tr('/', '/')
    end

    #: (Hash[String, Feature]) -> void
    def resolve_all_imports(features)
      resolved = Set.new
      features.each_key do |name|
        feature = features[name]
        next unless feature.imports

        resolve_imports_for_feature(name, features, Set.new, resolved)
      end
    end

    #: (String, Hash[String, Feature], Set[String], Set[String]) -> Hash[String, untyped]
    def resolve_imports_for_feature(feature_name, features, resolution_path, resolved)
      feature = features[feature_name]

      # If already resolved, return a deep clone of the (possibly merged) options
      return deep_clone(feature.options) if resolved.include?(feature_name)

      # If no imports, mark as resolved and return a deep clone
      unless feature&.imports
        resolved.add(feature_name)
        return deep_clone(feature.options)
      end

      # Check for cycles
      raise "Cycle detected when resolving imports for '#{feature_name}'. Path: #{resolution_path.to_a}" if resolution_path.include?(feature_name)

      # Add current feature to resolution path
      new_path = resolution_path.dup.add(feature_name)

      # Build merged options from imports
      merged_options = {}
      feature.imports&.each do |import_name|
        imported_feature = features[import_name]
        raise "Import '#{import_name}' not found for feature '#{feature_name}'" unless imported_feature

        # Check that imported feature doesn't have conditions
        raise "Import '#{import_name}' has conditions. Imported features cannot have conditions." if imported_feature.conditions

        # Recursively resolve imports for the imported feature
        import_options = resolve_imports_for_feature(import_name, features, new_path, resolved)
        deep_merge!(merged_options, import_options)
      end

      # Merge feature's own options on top (deep clone to avoid shared references)
      deep_merge!(merged_options, deep_clone(feature.options))

      # Debug output
      puts "[DEBUG] Resolved imports for '#{feature_name}': #{merged_options.inspect[0..200]}" if ENV['DEBUG_IMPORTS']

      # Update the feature's options with merged result and mark as resolved
      if ENV['DEBUG_IMPORTS']
        old_opts = feature.options
        puts "[DEBUG] Updating '#{feature_name}' options from #{old_opts.dig('message', 'arguments',
                                                                             'name').inspect} to #{merged_options.dig('message', 'arguments', 'name').inspect}"
      end
      feature.instance_variable_set(:@options, merged_options)
      resolved.add(feature_name)

      # Return a deep clone to avoid shared references
      deep_clone(merged_options)
    end

    #: (Hash[String, untyped], Hash[String, untyped]) -> Hash[String, untyped]
    def deep_merge!(target, source)
      source.each do |key, value|
        if value.is_a?(Hash) && target[key].is_a?(Hash)
          deep_merge!(target[key], value)
        elsif value.is_a?(Array) && target[key].is_a?(Array)
          target[key] = value
        else
          target[key] = value
        end
      end
      target
    end

    #: (untyped) -> untyped
    def deep_clone(obj)
      case obj
      when Hash
        obj.transform_values { |v| deep_clone(v) }
      when Array
        obj.map { |v| deep_clone(v) }
      else
        obj
      end
    end
  end
end
