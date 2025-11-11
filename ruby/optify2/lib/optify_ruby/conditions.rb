# typed: strict
# frozen_string_literal: true

require 'sorbet-runtime'
require 'json'

module Optify
  # Evaluates conditions for feature filtering based on constraints
  module Conditions
    extend T::Sig

    #: (Hash[String, untyped]? conditions, Hash[String, untyped] constraints) -> bool
    def self.evaluate(conditions, constraints)
      return true if conditions.nil? || conditions.empty?

      evaluate_condition(conditions, constraints)
    end

    #: (Hash[String, untyped] condition, Hash[String, untyped] constraints) -> bool
    def self.evaluate_condition(condition, constraints)
      if condition.key?('and')
        evaluate_and(condition['and'], constraints)
      elsif condition.key?('or')
        evaluate_or(condition['or'], constraints)
      elsif condition.key?('not')
        !evaluate_condition(condition['not'], constraints)
      elsif condition.key?('jsonPointer')
        evaluate_json_pointer(condition, constraints)
      elsif condition.key?('in')
        evaluate_in(condition, constraints)
      else
        true
      end
    end

    #: (Array[Hash[String, untyped]] conditions, Hash[String, untyped] constraints) -> bool
    def self.evaluate_and(conditions, constraints)
      conditions.all? { |cond| evaluate_condition(cond, constraints) }
    end

    #: (Array[Hash[String, untyped]] conditions, Hash[String, untyped] constraints) -> bool
    def self.evaluate_or(conditions, constraints)
      conditions.any? { |cond| evaluate_condition(cond, constraints) }
    end

    #: (Hash[String, untyped] condition, Hash[String, untyped] constraints) -> bool
    def self.evaluate_json_pointer(condition, constraints)
      pointer = condition['jsonPointer']
      value = resolve_json_pointer(pointer, constraints)

      if condition.key?('equals')
        value == condition['equals']
      elsif condition.key?('matches')
        pattern = Regexp.new(condition['matches'])
        value_str = value.is_a?(String) ? value : JSON.generate(value)
        pattern.match?(value_str)
      elsif condition.key?('in')
        array = condition['in']
        array.include?(value)
      else
        false
      end
    end

    #: (Hash[String, untyped] condition, Hash[String, untyped] constraints) -> bool
    def self.evaluate_in(condition, constraints)
      pointer = condition['in']
      array = resolve_json_pointer(pointer, constraints)
      return false unless array.is_a?(Array)

      if condition.key?('equals')
        array.include?(condition['equals'])
      elsif condition.key?('matches')
        pattern = Regexp.new(condition['matches'])
        array.any? do |item|
          item_str = item.is_a?(String) ? item : JSON.generate(item)
          pattern.match?(item_str)
        end
      else
        false
      end
    end

    #: (String pointer, Hash[String, untyped] data) -> untyped
    def self.resolve_json_pointer(pointer, data)
      return data if ['', '/'].include?(pointer)

      parts = pointer.split('/').drop(1)
      current = data

      parts.each do |part|
        part = part.gsub('~1', '/')
        part.gsub!('~0', '~')

        if current.is_a?(Hash)
          current = current[part]
        elsif current.is_a?(Array)
          index = part.to_i
          current = current[index]
        else
          return nil
        end

        return nil if current.nil?
      end

      current
    end
  end
end
