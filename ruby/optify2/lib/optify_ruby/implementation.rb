# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require_relative 'base_config'
require_relative 'options_metadata'
require_relative 'provider_module'
require_relative 'options_provider_impl'
require_relative 'builder'

module Optify
  # Options for caching
  class CacheOptions < FromHashable
  end

  # Provides configurations based on keys and enabled feature names
  class OptionsProvider
    include ProviderModule

    #: -> void
    def initialize
      raise 'Do not instantiate OptionsProvider directly. Use OptionsProviderBuilder instead.'
    end

    #: ((OptionsProviderImpl | OptionsWatcherImpl) impl) -> OptionsProvider
    def self.from_impl(impl)
      provider = allocate
      provider.instance_variable_set(:@impl, impl)
      provider.instance_variable_set(:@cache, nil)
      provider.instance_variable_set(:@features_with_metadata, nil)
      provider
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

    #: (Array[String] feature_names, GetOptionsPreferences preferences) -> String
    def get_all_options_json(feature_names, preferences)
      impl.get_all_options_json(feature_names, preferences)
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

    #: (Array[String] feature_names, GetOptionsPreferences preferences) -> Array[String]
    def get_filtered_features(feature_names, preferences)
      impl.get_filtered_features(feature_names, preferences)
    end

    #: [Config]
    #| (String, Array[String], Class[Config], ?CacheOptions?, ?Optify::GetOptionsPreferences?)
    #| -> Config
    def get_options(key, feature_names, config_class, cache_options = nil, preferences = nil)
      _get_options(key, feature_names, config_class, cache_options, preferences)
    end

    #: -> OptionsProvider
    def init
      _init
      self
    end

    private

    #: -> OptionsProviderImpl
    def impl
      @impl ||= nil #: OptionsProviderImpl?
      raise 'OptionsProvider not properly initialized' unless @impl

      @impl
    end
  end

  # Builder for OptionsProvider
  class OptionsProviderBuilder
    #: -> void
    def initialize
      @builder = OptionsProviderBuilderImpl.new #: OptionsProviderBuilderImpl
    end

    #: (String directory) -> OptionsProviderBuilder
    def add_directory(directory)
      @builder.add_directory(directory)
      self
    end

    #: -> OptionsProvider
    def build
      impl = @builder.build
      OptionsProvider.from_impl(impl)
    end
  end
end
