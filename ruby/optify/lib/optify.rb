# frozen_string_literal: true
# typed: strict

# The implementation to use directly Ruby and with types declared.
require_relative "optify_ruby/implementation"

puts "Loading Optify bindings. Warnings about redefining methods are normal and can be ignored because the implementations in Ruby are not implemented and only exist to help with type checking."

# The implementation in Rust which redefines some methods.
# This yields some warnings, but we should redeclare the methods in Ruby to help with type checking anyway.
require_relative "optify_ruby/optify_ruby"
