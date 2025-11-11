# typed: strict
# frozen_string_literal: true

# The implementation to use directly Ruby and with types declared.
require_relative 'optify_ruby/get_options_preferences'
require_relative 'optify_ruby/implementation'
require_relative 'optify_ruby/watcher_implementation'

# The implementation in Rust which redefines some methods.
# This yields some warnings, but we should redeclare the methods in Ruby to help with type checking anyway.
# Warnings about redefining methods are normal and can be ignored
# because the implementations in Ruby are not implemented and only exist to help with type checking.
# Ideally we do:
# `require_relative 'optify_ruby/optify_ruby'`
# but that doesn't work when building for multiple versions of Ruby.
# So we have to do this which is similar to something from 'https://github.com/matsadler/halton-rb/blob/main/lib/halton.rb'.
begin
  ruby_version = RUBY_VERSION.match(/\d+\.\d+/)&.[](0)
  require_relative "optify_ruby/#{ruby_version}/optify_ruby"
rescue LoadError
  begin
    require_relative 'optify_ruby/optify_ruby'
  rescue LoadError # Cargo Builder in RubyGems < 3.4.6 doesn't install to dir
    require_relative 'optify_ruby.so'
  end
end

require_relative 'optify_ruby/base_config'
