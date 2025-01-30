# typed: strict
# frozen_string_literal: true

require "test/unit"
require_relative "../lib/optify"

class SampleTest < Test::Unit::TestCase

  def test_dddristance
    assert { dddristance([1, 2], [1+3, 2+4]) == 5 }
  end

end
