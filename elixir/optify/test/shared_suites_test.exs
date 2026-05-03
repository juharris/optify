defmodule Optify.SharedSuitesTest do
  use ExUnit.Case, async: true

  import Optify.TestHelpers

  alias Optify.GetOptionsPreferences
  alias Optify.OptionsProvider

  for suite <- File.ls!(suites_root()) do
    configs_path = suite_configs(suite)
    expectations_path = suite_expectations(suite)

    for expectation_file <- File.ls!(expectations_path) do
      expectation_path = Path.join(expectations_path, expectation_file)
      expected_info = Jason.decode!(File.read!(expectation_path))
      features = Macro.escape(expected_info["features"])
      constraints = Macro.escape(expected_info["constraints"])

      prefs_setup =
        if expected_info["constraints"] do
          quote do
            GetOptionsPreferences.new()
            |> GetOptionsPreferences.enable_configurable_strings()
            |> GetOptionsPreferences.set_constraints(unquote(constraints))
          end
        else
          quote do
            GetOptionsPreferences.new() |> GetOptionsPreferences.enable_configurable_strings()
          end
        end

      for {key, expected_value} <- expected_info["options"] do
        escaped_key = key
        escaped_expected_value = Macro.escape(expected_value)

        test "suite #{suite} / #{expectation_file} / #{key}" do
          provider = OptionsProvider.build!(unquote(configs_path))
          prefs = unquote(prefs_setup)

          options =
            OptionsProvider.get_options(
              provider,
              unquote(escaped_key),
              unquote(features),
              prefs
            )

          assert options == unquote(escaped_expected_value)
        end
      end
    end
  end
end
