# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require 'json'
require_relative 'feature'
require_relative 'configurable_string'

module Optify
  # Core implementation of OptionsProvider in pure Ruby
  class OptionsProviderImpl
    #: Hash[String, Feature]
    attr_reader :features

    #: Hash[String, String]
    attr_reader :alias_map

    #: Array[String]
    attr_reader :config_directories

    #: (
    #|   Hash[String, Feature] features,
    #|   Hash[String, String] alias_map,
    #|   Array[String] config_directories,
    #|   bool are_configurable_strings_enabled,
    #|   Hash[String, String] loaded_files
    #| ) -> void
    def initialize(features, alias_map, config_directories, are_configurable_strings_enabled, loaded_files)
      @features = features #: Hash[String, Feature]
      @alias_map = alias_map #: Hash[String, String]
      @config_directories = config_directories #: Array[String]
      @are_configurable_strings_enabled = are_configurable_strings_enabled #: bool
      @loaded_files = loaded_files #: Hash[String, String]
      @cache = nil #: Hash[untyped, untyped]?
      @features_with_metadata = nil #: Hash[String, OptionsMetadata]?
    end

    #: -> Array[String]
    def feature_names
      @features.keys.sort
    end

    #: -> Array[String]
    def alias_names
      # Return only actual aliases from feature metadata, not feature names
      @features.values.flat_map { |f| f.metadata.aliases }.compact.sort.uniq
    end

    #: -> Array[String]
    def features_and_aliases
      (feature_names + alias_names).sort.uniq
    end

    #: (String alias_name) -> String
    def get_canonical_feature_name(alias_name)
      canonical = @alias_map[alias_name.downcase]
      return canonical if canonical

      return alias_name if @features.key?(alias_name)

      raise "Feature name \"#{alias_name}\" is not a known feature."
    end

    #: (Array[String] feature_names) -> Array[String]
    def get_canonical_feature_names(feature_names)
      feature_names.map { |name| get_canonical_feature_name(name) }
    end

    #: (String canonical_feature_name) -> String?
    def get_feature_metadata_json(canonical_feature_name)
      feature = @features[canonical_feature_name]
      return nil unless feature

      feature.metadata.to_json
    end

    #: (String canonical_feature_name) -> OptionsMetadata?
    def get_feature_metadata(canonical_feature_name)
      feature = @features[canonical_feature_name]
      return nil unless feature

      feature.metadata
    end

    #: -> String
    def features_with_metadata_json
      features_with_metadata.to_json
    end

    #: -> Hash[String, OptionsMetadata]
    def features_with_metadata
      return @features_with_metadata if @features_with_metadata

      result = {}
      @features.each do |name, feature|
        result[name] = feature.metadata
      end
      @features_with_metadata = result
      result
    end

    #: (String key, Array[String] feature_names) -> String
    def get_options_json(key, feature_names)
      # Canonicalize feature names (aliases to feature names)
      canonical_names = get_canonical_feature_names(feature_names)
      options = build_options(key, canonical_names, {}, false)
      JSON.generate(options)
    end

    #: (String key, Array[String] feature_names, GetOptionsPreferences preferences) -> String
    def get_options_json_with_preferences(key, feature_names, preferences)
      skip_conversion = preferences.skip_feature_name_conversion
      constraints = preferences.constraints || {}
      overrides = preferences.overrides || {}
      # Builder config acts as a gate - even if preference enables it, builder must allow it
      # Preference can enable it only if builder allows, or disable it regardless
      configurable_strings_enabled = if preferences.configurable_strings_explicitly_set
                                       # Preference was set explicitly
                                       preferences.are_configurable_strings_enabled? && @are_configurable_strings_enabled
                                     else
                                       # Use builder default
                                       @are_configurable_strings_enabled
                                     end

      canonical_names = if skip_conversion
                          # Validate that all feature names are valid when skipping conversion
                          feature_names.each do |name|
                            raise "Feature name \"#{name}\" is not a known feature." unless @features.key?(name)
                          end
                          feature_names
                        else
                          get_canonical_feature_names(feature_names)
                        end

      filtered_names = filter_features_by_constraints(canonical_names, constraints)

      options = build_options(key, filtered_names, overrides, configurable_strings_enabled)

      JSON.generate(options)
    end

    #: (Array[String] feature_names, GetOptionsPreferences preferences) -> Array[String]
    def get_filtered_features(feature_names, preferences)
      skip_conversion = preferences.skip_feature_name_conversion
      constraints = preferences.constraints || {}

      canonical_names = if skip_conversion
                          feature_names
                        else
                          get_canonical_feature_names(feature_names)
                        end

      filter_features_by_constraints(canonical_names, constraints)
    end

    #: (Array[String] feature_names, GetOptionsPreferences preferences) -> String
    def get_all_options_json(feature_names, preferences)
      skip_conversion = preferences.skip_feature_name_conversion
      # FIXME: Handle nils instead of creating empty hashes.
      constraints = preferences.constraints || {}
      overrides = preferences.overrides || {}
      configurable_strings_enabled = preferences.are_configurable_strings_enabled?

      canonical_names = if skip_conversion
                          feature_names
                        else
                          get_canonical_feature_names(feature_names)
                        end

      filtered_names = filter_features_by_constraints(canonical_names, constraints)

      all_keys = Set.new
      filtered_names.each do |name|
        feature = @features[name]
        all_keys.merge(feature.options.keys) if feature
      end

      # Also include keys from overrides
      all_keys.merge(overrides.keys)

      result = {}
      all_keys.each do |key|
        options = build_options(key, filtered_names, overrides, configurable_strings_enabled)
        result[key] = options
      end

      JSON.generate(result)
    end

    private

    #: (Array[String] canonical_names, Hash[String, untyped] constraints) -> Array[String]
    def filter_features_by_constraints(canonical_names, constraints)
      return canonical_names if constraints.empty?

      canonical_names.select do |name|
        feature = @features[name]
        next true unless feature

        conditions = feature.conditions
        next true if conditions.nil?

        conditions.evaluate(constraints)
      end
    end

    #: (
    #|   String key,
    #|   Array[String] feature_names,
    #|   Hash[String, untyped] overrides,
    #|   bool configurable_strings_enabled
    #| ) -> Hash[String, untyped]
    def build_options(key, feature_names, overrides, configurable_strings_enabled)
      # Handle nested keys like "myConfig.rootString"
      key_parts = key.split('.')
      root_key = key_parts.first
      # FIXME: Don't return empty, instead raise an error because the key was not found since the key was not found... actually how does this even happen?
      return {} unless root_key

      result = {} #: untyped

      # Check if key exists in any feature or overrides
      key_found_in_override = overrides.key?(root_key) && !overrides[root_key].nil?
      key_found_in_any_feature = feature_names.any? do |name|
        feature = @features[name]
        feature&.options&.key?(root_key)
      end

      unless key_found_in_any_feature || key_found_in_override
        raise ArgumentError, "Error getting options with features #{feature_names}: missing configuration field \"#{root_key}\""
      end

      feature_names.each do |name|
        feature = @features[name]
        next unless feature

        feature_options = feature.options[root_key]
        next unless feature_options

        result = merge_feature_options(result, feature_options)
      end

      # Extract overrides for the root key (if overrides are keyed by config key)
      # Overrides can be in format: { 'myConfig' => { ... } } or just { ... }
      key_overrides = overrides[root_key] || {}
      deep_merge!(result, key_overrides) if !key_overrides.empty? && result.is_a?(Hash)

      result = ConfigurableString.process_value(result, @loaded_files) if configurable_strings_enabled

      # Navigate to nested value if key has path
      if key_parts.length > 1
        key_rest = key_parts[1..]
        return {} unless key_rest

        key_rest.each do |part|
          result = result.is_a?(Hash) ? result[part] : nil
          break if result.nil?
        end
        result || {}
      else
        result
      end
    end

    #: (Hash[String, untyped] target, Hash[String, untyped] source) -> Hash[String, untyped]
    def deep_merge!(target, source)
      source.each do |key, value|
        if value.is_a?(Hash) && target[key].is_a?(Hash)
          deep_merge!(target[key], value)
        elsif value.is_a?(Array)
          # Deep clone arrays to avoid shared references
          target[key] = value.map { |v| v.is_a?(Hash) || v.is_a?(Array) ? deep_clone_value(v) : v }
        elsif value.is_a?(Hash)
          # Deep clone hashes to avoid shared references
          target[key] = deep_clone_value(value)
        else
          target[key] = value
        end
      end
      target
    end

    #: (untyped) -> untyped
    def deep_clone_value(obj)
      case obj
      when Hash
        obj.transform_values { |v| deep_clone_value(v) }
      when Array
        obj.map { |v| deep_clone_value(v) }
      else
        obj
      end
    end

    #: (untyped result, untyped feature_options) -> untyped
    def merge_feature_options(result, feature_options)
      if feature_options.is_a?(Hash)
        deep_merge!(result, feature_options)
        result
      else
        feature_options
      end
    end
  end
end
