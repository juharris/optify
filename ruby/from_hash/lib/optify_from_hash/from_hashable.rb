# typed: strict
# frozen_string_literal: true

require 'json'
require 'set'
require 'sorbet-runtime'
require 'tapioca'

module Optify
  # A base class for classes that can be created from a hash.
  class FromHashable
    extend T::Sig
    extend T::Helpers
    abstract!

    @return_type_cache = {} #: Hash[Symbol, T::Types::Base]

    # class << self
    #   #: Hash[Symbol, T::Types::Base]
    #   attr_reader :return_type_cache
    # end

    #: [T < Optify::FromHashable] (Class[T]) -> void
    def self.inherited(subclass)
      super

      # Trace the execution after the subclass finishes loading to capture its methods
      TracePoint.trace(:end) do |tp|
        if tp.self == subclass
          puts "Static setup: #{subclass} has just inherited #{self}"
          # TODO: Try to re-use the once already initialized, maybe.
          return_type_cache = {}
          subclass.public_instance_methods(false).each do |method_name|
            method = subclass.instance_method(method_name)
            sig = T::Utils.signature_for_method(method)
            next if sig.nil?

            return_type = sig.return_type
            puts "#{subclass}.#{method_name} has return type: #{return_type}"
            return_type_cache[method_name] = return_type
          end

          subclass.class_eval do
            @return_type_cache = return_type_cache

            # Create a singleton reader method specifically for this child class
            class << self
              #: Hash[Symbol, T::Types::Base]
              attr_reader :return_type_cache
            end
          end

          tp.disable
        end
      end
    end

    # Create a new immutable instance of the class from a hash.
    #
    # @param hash The hash to create the instance from.
    # @return The new instance.
    #: (Hash[untyped, untyped]) -> instance
    def self.from_hash(hash)
      instance = new

      hash.each do |key, value|
        # begin
        #   method = instance_method(key)
        # rescue StandardError
        #   raise ArgumentError,
        #         "Error converting hash to `#{name}` because of key \"#{key}\". Perhaps \"#{key}\" is not a valid attribute for `#{name}`."
        # end

        # sig = T::Utils.signature_for_method(method)
        # raise "A Sorbet signature is required for `#{name}.#{key}`." if sig.nil?

        # sig_return_type = sig.return_type
        # puts "#{name}.#{key}: Getting return type from signature cache: #{@signature_cache}"
        sig_return_type = @return_type_cache.fetch(key) do
          raise "A Sorbet signature is required for `#{name}.#{key}`."
        end
        value = _convert_value(value, sig_return_type)
        instance.instance_variable_set("@#{key}", value)
      end

      instance.freeze
    end

    #: (Array[untyped], untyped) -> (Array[untyped] | Set[untyped])
    def self._convert_array(value, unwrapped_type)
      inner_type = unwrapped_type.type
      return value.map { |v| _convert_value(v, inner_type) }.freeze if unwrapped_type.is_a?(T::Types::TypedArray)

      value.each_with_object(Set.new) { |v, set| set.add(_convert_value(v, inner_type)) }.freeze
    end

    #: (untyped, T::Types::Base) -> untyped
    def self._convert_value(value, type)
      if type.is_a?(T::Types::Untyped)
        # No preferred type is given, so return the value as is.
        return value
      end

      unwrapped_type = _unwrap_nilable(type)
      return value&.to_sym if unwrapped_type.is_a?(T::Types::Simple) && unwrapped_type.raw_type == Symbol

      case value
      when Array
        return _convert_array(value, unwrapped_type)
      when Hash
        # Handle `T.nilable(T::Hash[...])` and `T.any(...)`.
        # We used to use `type = type.unwrap_nilable if type.respond_to?(:unwrap_nilable)`, but it's not needed now that we handle
        # `T.any(...)` because using `.types` works for both cases.
        if type.respond_to?(:types)
          # Find a type that works for the hash.
          type #: as untyped
            .types.each do |t|
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

    #: (Hash[untyped, untyped], T::Types::Base) -> untyped
    def self._convert_hash(hash, type)
      if type.respond_to?(:raw_type)
        # There is an object for the hash.
        # It could be a custom class, a String, or maybe something else.
        type_for_hash = type #: as untyped
                        .raw_type
        return type_for_hash.from_hash(hash) if type_for_hash.respond_to?(:from_hash)
      elsif type.is_a?(T::Types::TypedHash)
        # The hash should be a hash, but the values might be objects to convert.
        type_for_keys = type.keys
        type_for_values = type.values

        result = hash
                 .transform_values { |v| _convert_value(v, type_for_values) }

        return result.transform_keys!(&:to_sym) if type_for_keys.is_a?(T::Types::Simple) && type_for_keys.raw_type == Symbol

        return result
      end

      raise TypeError, "Could not convert hash #{hash} to `#{type}`."
    end

    # Unwrap `T.nilable(...)` to get the inner type, or return the type as-is.
    #: (T::Types::Base) -> T::Types::Base
    def self._unwrap_nilable(type)
      if type.respond_to?(:unwrap_nilable)
        type #: as untyped
          .unwrap_nilable
      else
        type
      end
    end

    private_class_method :_convert_array, :_convert_hash, :_convert_value, :_unwrap_nilable

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

    # Support equality by value so that instances can be used in Sets and as Hash keys.
    #: (untyped) -> bool
    def eql?(other)
      return true if other.equal?(self)
      return false if self.class != other.class

      instance_variables.all? do |name|
        instance_variable_get(name).eql?(other.instance_variable_get(name))
      end
    end

    # @return [Integer] a hash value based on the object's class and instance variables.
    #: () -> Integer
    def hash
      [self.class, *instance_variables.sort.map { |name| instance_variable_get(name) }].hash
    end

    # Convert this object to a JSON string.
    #: (?JSON::State?) -> String
    def to_json(state = nil)
      to_h.to_json(state)
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
        result[method_name.to_sym] = self.class.send(:_convert_value_for_to_h, value)
      end

      result
    end

    #: (untyped) -> untyped
    def self._convert_value_for_to_h(value)
      case value
      # Treat sets like arrays for JSON serialization.
      when Array, Set
        value.map { |v| _convert_value_for_to_h(v) }
      when Hash
        value.transform_values { |v| _convert_value_for_to_h(v) }
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

    private_class_method :_convert_value_for_to_h
  end
end
