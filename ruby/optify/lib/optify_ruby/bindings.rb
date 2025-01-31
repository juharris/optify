# frozen_string_literal: true
# typed: strict

require 'sorbet-runtime'

# This file helps with type checks for implementations in Rust.
module Optify
  class RustOptionsProviderBuilder
    extend T::Sig
    sig { params(path: String).returns(RustOptionsProviderBuilder) }
    def add_directory(path)
      self
    end

    sig { returns(Optify::RustOptionsProvider) }
    def build
      return RustOptionsProvider.new
    end
  end

  class RustOptionsProvider
    extend T::Sig
    sig { params(key: String, feature_names: T::Array[String]).returns(String) }
    def get_options(key, feature_names)
      return ""
    end
  end
end