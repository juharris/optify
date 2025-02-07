# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require 'tapioca'

module Optify
  # A base class for classes from configuration files.
  # Classes that derive from this can easily be used with `Optify::OptionsProvider.get_options`
  # because they will have an implementation of `from_hash` that works recursively.
  # This class is a work in progress with minimal error handling
  # and doesn't handle certain cases such as nilable types yet.
  # It may be moved to another gem in the future.
  class BaseConfig
    extend T::Sig
    extend T::Helpers
    abstract!

    # Create a new instance of the class from a hash.
    #
    # @param hash [Hash] The hash to create the instance from.
    # @return The new instance.
    sig { params(hash: T::Hash[T.untyped, T.untyped]).returns(T.attached_class) }
    def self.from_hash(hash)
      result = new

      hash.each do |key, value|
        # TODO: Might need some error handling here, but it should be fine if type signatures are used.
        # TODO Handle nilable types.
        case value
        when Array
          sig_return_type = T::Utils.signature_for_method(instance_method(key)).return_type
          inner_type = sig_return_type.type.raw_type
          value = value.map { |v| inner_type.from_hash(v) } if inner_type.respond_to?(:from_hash)
        when Hash
          sig_return_type = T::Utils.signature_for_method(instance_method(key)).return_type
          if sig_return_type.respond_to?(:raw_type)
            type_for_key = sig_return_type.raw_type
            value = type_for_key.from_hash(value) if type_for_key.respond_to?(:from_hash)
          end
        end

        result.instance_variable_set("@#{key}", value)
      end
      result
    end
  end
end
