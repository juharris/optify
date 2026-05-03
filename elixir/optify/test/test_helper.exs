ExUnit.start()

defmodule Optify.TestHelpers do
  @suites_root Path.expand("../../../tests/test_suites", __DIR__)
  @schema_path Path.expand("../../../schemas/feature_file.json", __DIR__)

  def simple_configs, do: suite_configs("simple")
  def conditions_configs, do: suite_configs("conditions")
  def schema_path, do: @schema_path
  def suites_root, do: @suites_root

  def suite_configs(name), do: Path.join([@suites_root, name, "configs"])
  def suite_expectations(name), do: Path.join([@suites_root, name, "expectations"])
  def missing_directory, do: Path.join(simple_configs(), "missing")

  def constraints_preferences(constraints) do
    Optify.GetOptionsPreferences.new()
    |> Optify.GetOptionsPreferences.enable_configurable_strings()
    |> Optify.GetOptionsPreferences.set_constraints(constraints)
  end
end
