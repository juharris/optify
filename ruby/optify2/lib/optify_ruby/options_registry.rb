# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require_relative 'provider_module'

module Optify
  # Base class for options registries
  class OptionsRegistry
    extend T::Sig
    extend T::Helpers
    include ProviderModule

    abstract!

    #: -> void
    def initialize
      raise 'Do not instantiate OptionsRegistry directly. Use a builder instead.'
    end

    #: -> Array[String]
    def features
      impl.feature_names
    end

    #: -> Array[String]
    def aliases
      impl.alias_names
    end

    #: -> Array[String]
    def features_and_aliases
      impl.features_and_aliases
    end

    #: (String alias_name) -> String
    def get_canonical_feature_name(alias_name)
      impl.get_canonical_feature_name(alias_name)
    end

    #: (Array[String]) -> Array[String]
    def _get_canonical_feature_names(feature_names)
      impl.get_canonical_feature_names(feature_names)
    end

    #: (String key, Array[String] feature_names) -> String
    def get_options_json(key, feature_names)
      impl.get_options_json(key, feature_names)
    end

    #: (String key, Array[String] feature_names, GetOptionsPreferences preferences) -> String
    def get_options_json_with_preferences(key, feature_names, preferences)
      impl.get_options_json_with_preferences(key, feature_names, preferences)
    end

    #: (String canonical_feature_name) -> String?
    def get_feature_metadata_json(canonical_feature_name)
      impl.get_feature_metadata_json(canonical_feature_name)
    end

    #: -> String
    def features_with_metadata_json
      impl.features_with_metadata_json
    end

    #: -> Hash[String, OptionsMetadata]
    def features_with_metadata
      _features_with_metadata
    end

    #: (Array[String] feature_names, GetOptionsPreferences preferences) -> String
    def get_all_options_json(feature_names, preferences)
      impl.get_all_options_json(feature_names, preferences)
    end

    #: (Array[String] feature_names, GetOptionsPreferences preferences) -> Array[String]
    def get_filtered_features(feature_names, preferences)
      impl.get_filtered_features(feature_names, preferences)
    end

    #: -> OptionsRegistry
    def init
      _init
      self
    end

    private

    #: -> (OptionsProviderImpl | OptionsWatcherImpl)
    def impl
      @impl ||= nil #: (OptionsProviderImpl | OptionsWatcherImpl)?
      raise 'OptionsRegistry not properly initialized' unless @impl

      @impl
    end
  end
end
