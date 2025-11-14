# typed: true
# frozen_string_literal: true

require 'sorbet-runtime'
require 'json'

module Optify
  # Represents a predicate for condition evaluation (equals or matches)
  class Predicate
    #: untyped
    attr_reader :value

    #: Symbol
    attr_reader :type

    #: (untyped value, Symbol type) -> void
    def initialize(value, type)
      @value = value
      @type = type
    end

    #: (untyped value) -> bool
    def evaluate(value)
      case @type
      when :equals
        @value == value
      when :matches
        value_str = value.is_a?(String) ? value : JSON.generate(value)
        @value.match?(value_str)
      else
        false
      end
    end

    #: (untyped value) -> Predicate
    def self.equals(value)
      new(value, :equals)
    end

    #: (String pattern) -> Predicate
    def self.matches(pattern)
      regex = Regexp.new(pattern)
      new(regex, :matches)
    end
  end

  # Represents a single condition with a JSON pointer and operator
  class Condition
    #: String
    attr_reader :json_pointer

    #: Predicate
    attr_reader :operator_value

    #: (String json_pointer, Predicate operator_value) -> void
    def initialize(json_pointer, operator_value)
      @json_pointer = json_pointer
      @operator_value = operator_value
    end

    #: (Hash[String, untyped] constraints) -> bool
    def evaluate(constraints)
      value = resolve_json_pointer(@json_pointer, constraints)
      return false if value.nil?

      @operator_value.evaluate(value)
    end

    private

    #: (String pointer, Hash[String, untyped] data) -> untyped
    def resolve_json_pointer(pointer, data)
      return data if ['', '/'].include?(pointer)

      parts = pointer.split('/').drop(1)
      resolve_pointer_parts(parts, data)
    end

    #: (Array[String] parts, untyped current) -> untyped
    def resolve_pointer_parts(parts, current)
      return current if parts.empty?

      first_part = parts.first
      return nil unless first_part

      part = first_part.gsub('~1', '/').gsub('~0', '~')

      next_value = if current.is_a?(Hash)
                     current[part]
                   elsif current.is_a?(Array)
                     index = part.to_i
                     current[index]
                   else
                     return nil
                   end

      return nil if next_value.nil?

      resolve_pointer_parts(parts.drop(1), next_value)
    end
  end

  # Represents a condition expression tree (Condition, And, Or, Not)
  class ConditionExpression
    #: (Condition | Array[ConditionExpression] | ConditionExpression)
    attr_reader :value

    #: Symbol
    attr_reader :type

    #: ((Condition | Array[ConditionExpression] | ConditionExpression) value, Symbol type) -> void
    def initialize(value, type)
      @value = value
      @type = type
    end

    #: (Hash[String, untyped] constraints) -> bool
    def evaluate(constraints)
      case @type
      when :condition
        @value.is_a?(Condition) ? @value.evaluate(constraints) : false
      when :and
        @value.is_a?(Array) ? @value.all? { |expr| expr.evaluate(constraints) } : false
      when :or
        @value.is_a?(Array) ? @value.any? { |expr| expr.evaluate(constraints) } : false
      when :not
        @value.is_a?(ConditionExpression) ? !@value.evaluate(constraints) : false
      else
        true
      end
    end

    #: (Condition condition) -> ConditionExpression
    def self.condition(condition)
      new(condition, :condition)
    end

    #: (Array[ConditionExpression] expressions) -> ConditionExpression
    def self.and(expressions)
      new(expressions, :and)
    end

    #: (Array[ConditionExpression] expressions) -> ConditionExpression
    def self.or(expressions)
      new(expressions, :or)
    end

    #: (ConditionExpression expression) -> ConditionExpression
    def self.not(expression)
      new(expression, :not)
    end

    # Parse a condition hash into a ConditionExpression tree
    #: (Hash[String, untyped] data) -> ConditionExpression
    def self.from_hash(data)
      # Check for jsonPointer first (matches Rust deserializer logic)
      if data.key?('jsonPointer')
        json_pointer = data['jsonPointer'].is_a?(String) ? data['jsonPointer'] : ''
        predicate = parse_predicate(data)
        condition = Condition.new(json_pointer, predicate)
        return ConditionExpression.condition(condition)
      end

      # Check for logical operators
      if data.key?('and')
        expressions = (data['and'].is_a?(Array) ? data['and'] : []).map { |expr| from_hash(expr.is_a?(Hash) ? expr : {}) }
        return ConditionExpression.and(expressions)
      end

      if data.key?('or')
        expressions = (data['or'].is_a?(Array) ? data['or'] : []).map { |expr| from_hash(expr.is_a?(Hash) ? expr : {}) }
        return ConditionExpression.or(expressions)
      end

      if data.key?('not')
        expression = from_hash(data['not'].is_a?(Hash) ? data['not'] : {})
        return ConditionExpression.not(expression)
      end

      raise 'Invalid condition expression'
    end

    #: (Hash[String, untyped] data) -> Predicate
    def self.parse_predicate(data)
      if data.key?('equals')
        Predicate.equals(data['equals'])
      elsif data.key?('matches')
        pattern = data['matches'].is_a?(String) ? data['matches'] : ''
        Predicate.matches(pattern)
      else
        raise 'Condition must have either "equals" or "matches" operator'
      end
    end
  end
end
