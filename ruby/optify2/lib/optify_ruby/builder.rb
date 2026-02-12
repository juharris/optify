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
    #: -> void
    def initialize
      @directories = [] #: Array[String]
      @features = {} #: Hash[String, Feature]
      @alias_map = {} #: Hash[String, String]
      @loaded_files = {} #: Hash[String, String]
      @builder_options = BuilderOptions.new #: BuilderOptions
      @has_loaded_config = false #: bool
    end

    #: (String) -> OptionsProviderBuilderImpl
    def add_directory(directory)
      # Load the directory immediately and read its config
      @directories << directory

      # Load config from this directory if we haven't loaded one yet
      unless @has_loaded_config
        config_path = File.join(directory, '.optify', 'config.json')
        if File.exist?(config_path)
          @builder_options = BuilderOptions.from_file(config_path)
          @has_loaded_config = true
        end
      end

      # Load all features from this directory
      load_directory(directory, @features, @alias_map, @loaded_files)

      self
    end

    #: -> OptionsProviderImpl
    def build
      # Resolve imports for all features (already loaded in add_directory)
      resolve_all_imports(@features)

      build_dependents_graph(@features)

      OptionsProviderImpl.new(
        @features,
        @alias_map,
        @directories,
        @builder_options.are_configurable_strings_enabled,
        @loaded_files
      )
    end

    private

    #: (String, Hash[String, Feature], Hash[String, String], Hash[String, String]) -> void
    def load_directory(directory, features, alias_map, loaded_files)
      return unless Dir.exist?(directory)

      # Load all files from directory (not just config files)
      Dir.glob(File.join(directory, '**', '*')).each do |file_path|
        next unless File.file?(file_path)
        next if file_path.include?('/.optify/')

        # Store file contents for configurable string resolution
        # FIXME: Don't store option files.
        begin
          relative_path = file_path.sub("#{directory}/", '')
          contents = File.read(file_path)
          loaded_files[relative_path] = contents
        rescue StandardError => e
          # FIXME: Give error.
          warn "Error reading file #{file_path}: #{e.message}. Skipping file."
        end
      end

      # Load features from config files
      Dir.glob(File.join(directory, '**', '*.{json,json5,yaml,yml}')).each do |file_path|
        next if file_path.include?('/.optify/')

        feature_name = extract_feature_name(file_path, directory)
        next unless feature_name

        begin
          feature = Feature.from_file(file_path, feature_name)
        rescue StandardError => e
          # Re-raise with more context about which file failed
          raise ArgumentError, "Error loading file '#{file_path}': #{e.message}"
        end
        features[feature_name] = feature

        # Add actual aliases to the map with duplicate checking
        feature.metadata.aliases&.each do |alias_name|
          lowercase_alias = alias_name.downcase
          lowercase_feature = feature_name.downcase

          # Check if alias is the same as the feature name
          if lowercase_alias == lowercase_feature
            raise ArgumentError, "The alias '#{alias_name}' for canonical feature name '#{feature_name}' is already mapped to '#{feature_name}'."
          end

          if alias_map.key?(lowercase_alias)
            existing = alias_map[lowercase_alias]
            raise ArgumentError, "The alias '#{alias_name}' for canonical feature name '#{feature_name}' is already mapped to '#{existing}'."
          end
          alias_map[lowercase_alias] = feature_name
        end
      end

      # Add feature names themselves for case-insensitive lookup with duplicate checking
      features.each_key do |fname|
        lowercase_fname = fname.downcase
        if alias_map.key?(lowercase_fname) && alias_map[lowercase_fname] != fname
          existing = alias_map[lowercase_fname]
          raise ArgumentError, "The alias '#{fname}' for canonical feature name '#{fname}' is already mapped to '#{existing}'."
        end
        alias_map[lowercase_fname] = fname
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
        next unless feature&.imports

        resolve_imports_for_feature(name, features, Set.new, resolved)
      end
    end

    #: (Hash[String, Feature]) -> void
    def build_dependents_graph(features)
      # Build a map of which features depend on each feature
      dependents_map = {} #: Hash[String, Array[String]]

      # First pass: build the dependents map
      features.each do |name, feature|
        imports = feature.imports
        next unless imports

        imports.each do |imported_name|
          dependents_map[imported_name] ||= []
          list = dependents_map[imported_name]
          next unless list

          list << name unless list.include?(name)
        end
      end

      # Second pass: update each feature's metadata with its dependents
      # rubocop:disable Style/CombinableLoops
      features.each do |name, feature|
        dependents = dependents_map[name]
        next if dependents.nil? || dependents.empty?

        # Create a new metadata hash with the dependents
        metadata_hash = {
          'name' => feature.metadata.name,
          'path' => feature.metadata.path,
          'aliases' => feature.metadata.aliases,
          'dependents' => dependents.sort,
          'details' => feature.metadata.details,
          'owners' => feature.metadata.owners
        }

        new_metadata = OptionsMetadata.from_hash(metadata_hash)
        feature.instance_variable_set(:@metadata, new_metadata)
      end
      # rubocop:enable Style/CombinableLoops
    end

    #: (String, Hash[String, Feature], Set[String], Set[String]) -> Hash[String, untyped]
    def resolve_imports_for_feature(feature_name, features, resolution_path, resolved)
      feature = features[feature_name]
      return {} unless feature

      # If already resolved, return a deep clone of the (possibly merged) options
      return deep_clone(feature.options) if resolved.include?(feature_name)

      # If no imports, mark as resolved and return a deep clone
      unless feature.imports
        resolved.add(feature_name)
        return deep_clone(feature.options)
      end

      # Check for cycles
      if resolution_path.include?(feature_name)
        raise "Error when resolving imports for '#{feature_name}': Cycle detected with import '#{feature_name}'. Path: #{resolution_path.to_a}"
      end

      # Add current feature to resolution path
      new_path = resolution_path.dup.add(feature_name)

      # Build merged options from imports
      merged_options = {}
      feature.imports&.each do |import_name|
        imported_feature = features[import_name]
        raise "Import '#{import_name}' not found for feature '#{feature_name}'" unless imported_feature

        # Check that imported feature doesn't have conditions
        unless imported_feature.conditions.nil?
          raise "Error when resolving imports for '#{feature_name}': The import '#{import_name}' has conditions. Conditions cannot be used in imported features."
        end

        # Recursively resolve imports for the imported feature
        import_options = resolve_imports_for_feature(import_name, features, new_path, resolved)
        deep_merge!(merged_options, import_options)
      end

      # Merge feature's own options on top (deep clone to avoid shared references)
      deep_merge!(merged_options, deep_clone(feature.options))

      # Update the feature's options with merged result and mark as resolved
      feature.instance_variable_set(:@options, merged_options)
      resolved.add(feature_name)

      # Return a deep clone to avoid shared references
      deep_clone(merged_options)
    end

    #: (Hash[String, untyped], Hash[String, untyped]) -> Hash[String, untyped]
    def deep_merge!(target, source)
      source.each do |key, value|
        case value
        when Hash
          if target[key].is_a?(Hash)
            deep_merge!(target[key], value)
          else
            target[key] = value
          end
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
