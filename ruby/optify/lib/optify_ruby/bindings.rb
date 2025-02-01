# frozen_string_literal: true
# typed: strict

require 'sorbet-runtime'

# This module exists to help with type checking and code completion.
# The real implementation is in a Rust library.
module OptifyBindings
  class OptionsProvider
    extend T::Sig
    sig { params(key: String, feature_names: T::Array[String]).returns(String) }
    def get_options(key, feature_names)
      raise NotImplementedError
    end
  end

  class OptionsProviderBuilder
    extend T::Sig
    sig { params(path: String).returns(OptionsProviderBuilder) }
    def add_directory(path)
      raise NotImplementedError
    end

    sig { returns(OptionsProvider) }
    def build
      raise NotImplementedError
    end
  end
end