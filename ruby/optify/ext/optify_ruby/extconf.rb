# frozen_string_literal: true
# typed: false

require 'mkmf'
require 'rb_sys/mkmf'

create_rust_makefile('optify_ruby/optify_ruby') do |r|
  r.profile = ENV.fetch('RB_SYS_CARGO_PROFILE', :dev).to_sym
  puts "RB_SYS_CARGO_PROFILE: #{r.profile}"
end
