# frozen_string_literal: true
# typed: strict

require 'test/unit'
require_relative '../lib/optify-from_hash'
require_relative 'my_config'

# Ensures that we can convert hashes to objects.
module FromHashTest
  class ToJsonTest < Test::Unit::TestCase
    #: -> void
    def test_to_json
      h = { rootString: 'hello', myObject: { two: 2 } }
      m = MyConfig.from_hash(h)
      actual = m.to_json
      assert_equal('{"rootString":"hello","myObject":{"two":2}}', actual)
      assert_equal(h.to_json, actual)
    end

    #: -> void
    def test_to_json_with_args
      h = { rootString: 'hello', myObject: { two: 2 } }
      m = MyConfig.from_hash(h)
      json_state = JSON::State.new(indent: '  ', space: ' ', object_nl: "\n")
      actual = m.to_json(json_state)
      assert_equal("{\n  \"rootString\": \"hello\",\n  \"myObject\": {\n    \"two\": 2\n  }\n}", actual)
      expected = h.to_json(json_state)
      assert_equal(expected, actual)
    end
  end
end
