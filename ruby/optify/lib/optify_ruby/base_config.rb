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

    # Create a new immutable instance of the class from a hash.
    #
    # This is a class method that so that it can set members with private setters.
    # @param hash The hash to create the instance from.
    # @return The new instance.
    #: (Hash[untyped, untyped] hash) -> instance
    def self.from_hash(hash)
      result = new

      hash.each do |key, value|
        sig_return_type = T::Utils.signature_for_method(instance_method(key)).return_type
        value = _convert_value(value, sig_return_type)
        result.instance_variable_set("@#{key}", value)
      end

      T.unsafe(result).freeze if T.unsafe(result).respond_to?(:freeze)
    end

    #: (untyped value, untyped type) -> untyped
    def self._convert_value(value, type)
      case value
      when Array
        # Handle `T.nilable(T::Array[...])`
        type = type.unwrap_nilable if type.respond_to?(:unwrap_nilable)
        inner_type = type.type
        return value.map { |v| _convert_value(v, inner_type) }.freeze
      when Hash
        # Handle `T.nilable(T::Hash[...])`
        type = type.unwrap_nilable if type.respond_to?(:unwrap_nilable)
        return _convert_hash(value, type).freeze
      end

      value
    end

    #: (Hash[untyped, untyped] hash, untyped type) -> untyped
    def self._convert_hash(hash, type) # rubocop:disable Metrics/PerceivedComplexity
      if type.respond_to?(:raw_type)
        # There is an object for the hash.
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
        end

        # The values are not recognized objects.
        return hash.transform_values { |v| _convert_value(v, type_for_values) }
      end

      # Fallback to doing nothing.
      # This can happen if there are is no type information for a key in the hash.
      hash
    end

    private_class_method :_convert_hash, :_convert_value
  end
end
