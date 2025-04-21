# frozen_string_literal: true
# typed: strict

#require 'sorbet-runtime'

module Optify
  module ProviderModule # rubocop:disable Style/Documentation
    extend T::Sig

    #: (Array[String] feature_names) -> Array[String]
    def get_canonical_feature_names(feature_names)
      # Try to optimize a typical case where there are just a few features.
      # Ideally in production, a single feature that imports many other features is used for the most common scenario.
      # Benchmarks show that it is faster to use a loop than to call the Rust implementation which involves making a `Vec<String>` and returning a `Vec<String>`.
      if feature_names.length < 4
        feature_names.map { |feature_name| get_canonical_feature_name(feature_name) }
      else
        _get_canonical_feature_names(feature_names)
      end
    end

    #: (String canonical_feature_name) -> Optify::OptionsMetadata?
    def get_feature_metadata(canonical_feature_name)
      metadata_json = get_feature_metadata_json(canonical_feature_name)
      return nil if metadata_json.nil?

      OptionsMetadata.from_hash(JSON.parse(metadata_json))
    end
  end
end
