# typed: strict
# frozen_string_literal: true

require 'optify-from_hash'
require 'sorbet-runtime'
require 'tapioca'

module Optify
  # DEPRECATED: Use `Optify::FromHashable` instead.
  # A base class for classes from configuration files.
  # Classes that derive from this can easily be used with `Optify::OptionsProvider.get_options`
  # because they will have an implementation of `from_hash` that works recursively.
  # This class is a work in progress with minimal error handling.
  # It may be moved to another gem in the future.
  class BaseConfig < Optify::FromHashable
    abstract!
  end
end
