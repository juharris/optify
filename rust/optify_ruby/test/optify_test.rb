# typed: strict
# frozen_string_literal: true

require "test/unit"
require_relative "../lib/optify"

class SampleTest < Test::Unit::TestCase

  def test_wtv
    assert { !"my value".empty? }
  end

end
