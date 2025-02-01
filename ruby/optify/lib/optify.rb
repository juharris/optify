# frozen_string_literal: true
# typed: strict

# A module to help with type checking:
require_relative "optify_ruby/bindings"

puts "Loading Optify bindings. Warnings about redefining methods are normal and can be ignored because the implementations in Ruby are not implemented and only exist to help with type checking."

# The implementation in Rust:
require_relative "optify_ruby/optify_ruby"

# The implementation to use in Ruby:
require_relative "optify_ruby/optify"
