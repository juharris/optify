# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require 'json'
require_relative 'feature'
require_relative 'conditions'
require_relative 'configurable_string'

module Optify
  # Core implementation of OptionsProvider in pure Ruby
  class OptionsProviderImpl
    extend T::Sig

    #: Hash[String, Feature]
    attr_reader :features

    #: Hash[String, String]
    attr_reader :aliases

    #: Array[String]
    attr_reader :config_directories

    #: (Hash[String, Feature] features, Hash[String, String] aliases, Array[String] config_directories) -> void
    def initialize(features, aliases, config_directories)
      @features = features
      @aliases = aliases
      @config_directories = config_directories
      @cache = nil #: Hash[untyped, untyped]?
      @features_with_metadata = nil #: Hash[String, OptionsMetadata]?
    end

    #: -> Array[String]
    def feature_names
      @features.keys.sort
    end

    #: -> Array[String]
    def alias_names
      @aliases.keys.sort
    end

    #: -> Array[String]
    def features_and_aliases
      (feature_names + alias_names).sort.uniq
    end

    #: (String alias_name) -> String
    def get_canonical_feature_name(alias_name)
      canonical = @aliases[alias_name.downcase]
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
      options = build_options(key, feature_names, {}, false)
      JSON.generate(options)
    end

    #: (String key, Array[String] feature_names, GetOptionsPreferences preferences) -> String
    def get_options_json_with_preferences(key, feature_names, preferences)
      skip_conversion = preferences.skip_feature_name_conversion
      constraints = preferences.constraints || {}
      overrides = preferences.overrides || {}
      configurable_strings_enabled = preferences.are_configurable_strings_enabled?

      canonical_names = if skip_conversion
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
      constraints = preferences.constraints || {}
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

      result = {}
      all_keys.each do |key|
        options = build_options(key, filtered_names, {}, configurable_strings_enabled)
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

        Conditions.evaluate(feature.conditions, constraints)
      end
    end

    #: (
    #|   String key,
    #|   Array[String] feature_names,
    #|   Hash[String, untyped] overrides,
    #|   bool configurable_strings_enabled
    #| ) -> Hash[String, untyped]
    def build_options(key, feature_names, overrides, configurable_strings_enabled)
      result = {}

      feature_names.each do |name|
        feature = @features[name]
        next unless feature

        feature_options = feature.options[key]
        next unless feature_options

        deep_merge!(result, feature_options)
      end

      deep_merge!(result, overrides) unless overrides.empty?

      if configurable_strings_enabled
        base_dir = @config_directories.first
        result = ConfigurableString.process_value(result, base_dir)
      end

      result
    end

    #: (Hash[String, untyped] target, Hash[String, untyped] source) -> Hash[String, untyped]
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
  end
end
