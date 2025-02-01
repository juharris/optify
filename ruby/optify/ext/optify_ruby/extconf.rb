# frozen_string_literal: true
# typed: false

require "mkmf"
require "rb_sys/mkmf"

# TODO See https://github.com/oxidize-rb/rb-sys/blob/main/gem/README.md about configuration and making a release build.
create_rust_makefile("optify_ruby/optify_ruby")
