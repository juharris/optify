# frozen_string_literal: true
# typed: strict

# The implementation to use directly Ruby and with types declared.
require_relative 'optify_ruby/implementation'

# The implementation in Rust which redefines some methods.
# This yields some warnings, but we should redeclare the methods in Ruby to help with type checking anyway.
# Warnings about redefining methods are normal and can be ignored
# because the implementations in Ruby are not implemented and only exist to help with type checking.
require_relative 'optify_ruby/optify_ruby'

require_relative 'optify_ruby/from_hashable'
