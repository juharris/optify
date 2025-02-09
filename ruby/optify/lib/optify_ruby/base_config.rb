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
    # This is a class method that so that it can set members with private setters.
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
          # TODO: Handle when the inner type is a hash and call convert_hash.
          value = value.map { |v| inner_type.from_hash(v) } if inner_type.respond_to?(:from_hash)
        when Hash
          sig_return_type = T::Utils.signature_for_method(instance_method(key)).return_type
          value = _convert_hash(value, sig_return_type)
        end

        result.instance_variable_set("@#{key}", value)
      end
      result
    end

    sig do
      params(
        hash: T::Hash[T.untyped, T.untyped],
        type: T.untyped
      ).returns(T.untyped)
    end
    def self._convert_hash(hash, type) # rubocop:disable Metrics/PerceivedComplexity
      if type.respond_to?(:raw_type)
        # When there is an object for the hash.
        type_for_hash = type.raw_type
        return type_for_hash.from_hash(hash) if type_for_hash.respond_to?(:from_hash)
      elsif type.instance_of?(T::Types::TypedHash)
        # The hash should be a hash, but the values might be objects to convert.
        type_for_values = type.values

        if type_for_values.respond_to?(:raw_type)
          raw_type_for_values = type_for_values.raw_type
          if raw_type_for_values.respond_to?(:from_hash)
            # Use proper types.
            return hash.transform_values { |v| raw_type_for_values.from_hash(v) }
          end
        elsif type_for_values.instance_of?(T::Types::TypedHash)
          # Recurse
          return hash.transform_values { |v| _convert_hash(v, type_for_values) }
        end
      end

      # Fallback to doing nothing.
      hash
    end

    private_class_method :_convert_hash
  end
end
