# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require 'tapioca'

module Optify
  # A base class for classes from configuration files.
  # Classes that derive from this can easily be used with `Optify::OptionsProvider.get_options`
  # because they will have an implementation of `from_hash` that works recursively.
  # This class is a work in progress with minimal error handling.
  # It may be moved to another gem in the future.
  class BaseConfig
    extend T::Sig
    extend T::Helpers
    abstract!

    # Create a new immutable instance of the class from a hash.
    #
    # This is a class method so that it can set members with private setters.
    # @param hash [Hash] The hash to create the instance from.
    # @return [instance] The new instance.
    #: (Hash[untyped, untyped] hash) -> instance
    def self.from_hash(hash)
      instance = new

      # Identify attribute-style reader methods defined on this concrete subclass.
      valid_attribute_methods = self.instance_methods(false)
        .reject { |m| m.to_s.end_with?('=') || [:==, :to_h].include?(m) }
        .sort

      hash.each do |raw_key, value|
        key = raw_key.is_a?(String) ? raw_key.to_sym : raw_key

        unless valid_attribute_methods.include?(key)
          provided_keys = hash.keys.map { |k| k.is_a?(String) ? k.to_sym : k }
          raise ArgumentError,
                "Unknown attribute `#{key}` for #{name}. " \
                "Valid attributes: #{valid_attribute_methods.join(', ')}. " \
                "Provided keys: #{provided_keys.join(', ')}."
        end

        method_sig = T::Utils.signature_for_method(instance_method(key))
        raise "A Sorbet signature is required for `#{name}##{key}`." if method_sig.nil?

        sig_return_type = method_sig.return_type
        coerced = _convert_value(value, sig_return_type)
        instance.instance_variable_set("@#{key}", coerced)
      end

      instance.freeze
    end

    #: (untyped value, untyped type) -> untyped
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
        if type.respond_to?(:types)
          # Find a type that works for the hash.
          type.types.each do |t|
            return _convert_hash(value, t).freeze
          rescue StandardError
            # Try next type.
          end
          raise TypeError, "Could not convert hash: #{value} to #{type}."
        end
        return _convert_hash(value, type).freeze
      end

      # Could add primitive type validation here in future.
      value
    end

    #: (Hash[untyped, untyped] hash, untyped type) -> untyped
    def self._convert_hash(hash, type)
      if type.respond_to?(:raw_type)
        # Could be a custom class, String, etc.
        type_for_hash = type.raw_type
        return type_for_hash.from_hash(hash) if type_for_hash.respond_to?(:from_hash)
      elsif type.is_a?(T::Types::TypedHash)
        # Convert a typed hash (possibly coercing keys and values).
        type_for_keys = type.keys
        convert_key =
          if type_for_keys.is_a?(T::Types::Simple) && type_for_keys.raw_type == Symbol
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
    #: (untyped other) -> bool
    def ==(other)
      return true if other.equal?(self)
      return false unless other.is_a?(self.class)

      instance_variables.all? do |name|
        instance_variable_get(name) == other.instance_variable_get(name)
      end
    end

    # Convert this object to a Hash recursively (symbol keys).
    #: () -> Hash[Symbol, untyped]
    def to_h
      result = Hash.new(instance_variables.size)
      instance_variables.each do |var_name|
        method_name = var_name.to_s[1..] # remove leading '@'
        value = instance_variable_get(var_name)
        result[method_name.to_sym] = _convert_value_to_hash(value)
      end
      result
    end

    private

    #: (untyped value) -> untyped
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
