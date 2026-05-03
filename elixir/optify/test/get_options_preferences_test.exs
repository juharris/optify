defmodule Optify.GetOptionsPreferencesTest do
  use ExUnit.Case, async: true

  alias Optify.GetOptionsPreferences

  test "configurable strings can be enabled and disabled" do
    prefs = GetOptionsPreferences.new()
    refute GetOptionsPreferences.are_configurable_strings_enabled?(prefs)

    prefs = GetOptionsPreferences.enable_configurable_strings(prefs)
    assert GetOptionsPreferences.are_configurable_strings_enabled?(prefs)

    prefs = GetOptionsPreferences.disable_configurable_strings(prefs)
    refute GetOptionsPreferences.are_configurable_strings_enabled?(prefs)
  end

  test "constraints and overrides round-trip through JSON getters" do
    prefs = GetOptionsPreferences.new()
    prefs = GetOptionsPreferences.set_constraints_json(prefs, ~s({"country":"US"}))
    assert GetOptionsPreferences.get_constraints_json(prefs) == ~s({"country":"US"})

    prefs =
      GetOptionsPreferences.set_overrides_json(
        prefs,
        ~s({"myConfig":{"rootString2":"override from json"}})
      )

    assert GetOptionsPreferences.get_overrides_json(prefs) ==
             ~s({"myConfig":{"rootString2":"override from json"}})
  end

  test "map overrides and feature name conversion flags round-trip" do
    prefs = GetOptionsPreferences.new()

    prefs =
      GetOptionsPreferences.set_overrides(prefs, %{
        "myConfig" => %{"rootString2" => "override from map"}
      })

    assert GetOptionsPreferences.get_overrides_json(prefs) ==
             ~s({"myConfig":{"rootString2":"override from map"}})

    refute GetOptionsPreferences.skip_feature_name_conversion?(prefs)
    prefs = GetOptionsPreferences.set_skip_feature_name_conversion(prefs, true)
    assert GetOptionsPreferences.skip_feature_name_conversion?(prefs)
  end
end
