# typed: strict
# frozen_string_literal: true

require "test/unit"
require_relative "../lib/optify"

class SampleTest < Test::Unit::TestCase

  def test_example
    builder = OptionsProviderBuilder.new
    assert_equal(3, builder.example)
  end

end
