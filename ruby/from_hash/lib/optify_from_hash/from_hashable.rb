# typed: strict
# frozen_string_literal: true

require 'json'
require 'sorbet-runtime'
require 'tapioca'

module Optify
  # A base class for classes that can be created from a hash.
  class FromHashable
    extend T::Sig
    extend T::Helpers
    abstract!

    # Create a new immutable instance of the class from a hash.
    #
    # This is a class method so that it can set members with private setters.
    # @param hash The hash to create the instance from.
    # @return The new instance.
    #: (Hash[untyped, untyped]) -> instance
    def self.from_hash(hash)
      instance = new

      hash.each do |key, value|
        begin
          method = instance_method(key)
        rescue StandardError
          raise ArgumentError,
                "Error converting hash to `#{name}` because of key \"#{key}\". Perhaps \"#{key}\" is not a valid attribute for `#{name}`."
        end

        sig = T::Utils.signature_for_method(method)
        raise "A Sorbet signature is required for `#{name}.#{key}`." if sig.nil?

        sig_return_type = sig.return_type
        value = _convert_value(value, sig_return_type)
        instance.instance_variable_set("@#{key}", value)
      end

      instance.freeze
    end

    #: (untyped, untyped) -> untyped
    def self._convert_value(value, type)
      if type.is_a?(T::Types::Untyped)
        # No preferred type is given, so return the value as is.
        return value
      end

      return value.to_sym if type.is_a?(T::Types::Simple) && type.raw_type == Symbol

      case value
      when Array
        # Handle `T.nilable(T::Array[...])`
        type = type.unwrap_nilable if type.respond_to?(:unwrap_nilable)
        inner_type = type.type
        return value.map { |v| _convert_value(v, inner_type) }.freeze
      when Hash
        # Handle `T.nilable(T::Hash[...])` and `T.any(...)`.
        # We used to use `type = type.unwrap_nilable if type.respond_to?(:unwrap_nilable)`, but it's not needed now that we handle
        # `T.any(...)` because using `.types` works for both cases.
        if type.respond_to?(:types)
          # Find a type that works for the hash.
          type.types.each do |t|
            return _convert_hash(value, t).freeze
          rescue StandardError
            # Ignore and try the next type.
          end
          raise TypeError, "Could not convert hash: #{value} to #{type}."
        end
        return _convert_hash(value, type).freeze
      end

      # It would be nice to validate that the value is of the correct type here.
      # For example that a string is a string and an Integer is an Integer.
      value
    end

    #: (Hash[untyped, untyped], untyped) -> untyped
    def self._convert_hash(hash, type)
      if type.respond_to?(:raw_type)
        # There is an object for the hash.
        # It could be a custom class, a String, or maybe something else.
        type_for_hash = type.raw_type
        return type_for_hash.from_hash(hash) if type_for_hash.respond_to?(:from_hash)
      elsif type.is_a?(T::Types::TypedHash)
        # The hash should be a hash, but the values might be objects to convert.
        type_for_keys = type.keys

        convert_key = if type_for_keys.is_a?(T::Types::Simple) && type_for_keys.raw_type == Symbol
                        lambda(&:to_sym)
                      else
                        lambda(&:itself)
                      end

        type_for_values = type.values
        return hash.map { |k, v| [convert_key.call(k), _convert_value(v, type_for_values)] }.to_h
      end

      raise TypeError, "Could not convert hash #{hash} to `#{type}`."
    end

    private_class_method :_convert_hash, :_convert_value

    # Compare this object with another object for equality.
    # @param other The object to compare.
    # @return [Boolean] true if the objects are equal; otherwise, false.
    #: (untyped) -> bool
    def ==(other)
      return true if other.equal?(self)
      return false unless other.is_a?(self.class)

      instance_variables.all? do |name|
        instance_variable_get(name) == other.instance_variable_get(name)
      end
    end

    # Convert this object to a JSON string.
    #: (*untyped) -> String
    def to_json(*args)
      to_h.to_json(args)
    end

    # Convert this object to a Hash recursively.
    # This is mostly the reverse operation of `from_hash`,
    # as keys will be symbols
    # and `from_hash` will convert strings to symbols if that's how the attribute is declared.
    # @return [Hash] The hash representation of this object.
    #: () -> Hash[Symbol, untyped]
    def to_h
      result = Hash.new(instance_variables.size)

      instance_variables.each do |var_name|
        # Remove the @ prefix to get the method name
        method_name = var_name.to_s[1..] #: as !nil
        value = instance_variable_get(var_name)
        result[method_name.to_sym] = _convert_value_to_hash(value)
      end

      result
    end

    private

    #: (untyped) -> untyped
    def _convert_value_to_hash(value)
      case value
      when Array
        value.map { |v| _convert_value_to_hash(v) }
      when Hash
        value.transform_values { |v| _convert_value_to_hash(v) }
      when nil
        nil
      else
        if value.respond_to?(:to_h)
          value.to_h
        else
          value
        end
      end
    end
  end
end
