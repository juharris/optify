defmodule Optify.OptionsProviderTest do
  use ExUnit.Case, async: true

  import Optify.TestHelpers

  alias Optify.GetOptionsPreferences
  alias Optify.OptionsProvider

  test "build APIs return providers and schema variants work" do
    assert {:ok, %OptionsProvider{} = provider} = OptionsProvider.build(simple_configs())
    assert %OptionsProvider{} = OptionsProvider.build!(simple_configs())

    assert {:ok, %OptionsProvider{} = schema_provider} =
             OptionsProvider.build_with_schema(simple_configs(), schema_path())

    assert %OptionsProvider{} =
             OptionsProvider.build_with_schema!(simple_configs(), schema_path())

    assert {:ok, %OptionsProvider{} = from_dirs_provider} =
             OptionsProvider.build_from_directories([simple_configs()])

    assert %OptionsProvider{} = OptionsProvider.build_from_directories!([simple_configs()])

    assert {:ok, %OptionsProvider{} = from_dirs_schema_provider} =
             OptionsProvider.build_from_directories_with_schema([simple_configs()], schema_path())

    assert %OptionsProvider{} =
             OptionsProvider.build_from_directories_with_schema!(
               [simple_configs()],
               schema_path()
             )

    assert Enum.sort(OptionsProvider.features(provider)) == [
             "A_with_comments",
             "feature_A",
             "feature_B/initial"
           ]

    assert Enum.sort(OptionsProvider.features(schema_provider)) ==
             OptionsProvider.features(provider) |> Enum.sort()

    assert Enum.sort(OptionsProvider.features(from_dirs_provider)) ==
             OptionsProvider.features(provider) |> Enum.sort()

    assert Enum.sort(OptionsProvider.features(from_dirs_schema_provider)) ==
             OptionsProvider.features(provider) |> Enum.sort()
  end

  test "build returns errors and bang variants raise on invalid directories" do
    assert {:error, _} = OptionsProvider.build(missing_directory())
    assert {:error, _} = OptionsProvider.build_with_schema(missing_directory(), schema_path())
    assert {:error, _} = OptionsProvider.build_from_directories([missing_directory()])

    assert {:error, _} =
             OptionsProvider.build_from_directories_with_schema(
               [missing_directory()],
               schema_path()
             )

    assert_raise ArgumentError, fn -> OptionsProvider.build!(missing_directory()) end

    assert_raise ArgumentError, fn ->
      OptionsProvider.build_with_schema!(missing_directory(), schema_path())
    end

    assert_raise ArgumentError, fn ->
      OptionsProvider.build_from_directories!([missing_directory()])
    end

    assert_raise ArgumentError, fn ->
      OptionsProvider.build_from_directories_with_schema!([missing_directory()], schema_path())
    end
  end

  test "build wraps native references and get_options passes through native errors" do
    native_provider_ref = Optify.Native.provider_build(simple_configs())

    assert is_reference(native_provider_ref)

    assert {:ok, %OptionsProvider{ref: wrapper_provider_ref}} =
             OptionsProvider.build(simple_configs())

    assert is_reference(wrapper_provider_ref)

    assert {:error,
            ~s(Error getting options with features ["feature_A"]: configuration property "unknown_key" not found)} =
             Optify.Native.provider_get_options(
               wrapper_provider_ref,
               "unknown_key",
               ["feature_A"],
               GetOptionsPreferences.new().ref
             )

    assert {:error,
            ~s(Error getting options with features ["feature_A"]: configuration property "unknown_key" not found)} =
             OptionsProvider.get_options(
               %OptionsProvider{ref: wrapper_provider_ref},
               "unknown_key",
               ["feature_A"]
             )
  end

  test "query helpers expose aliases, features, and canonical feature names" do
    provider = OptionsProvider.build!(simple_configs())

    assert Enum.sort(OptionsProvider.aliases(provider)) == ["a", "b"]

    assert Enum.sort(OptionsProvider.features(provider)) == [
             "A_with_comments",
             "feature_A",
             "feature_B/initial"
           ]

    assert Enum.sort(OptionsProvider.features_and_aliases(provider)) ==
             Enum.sort(OptionsProvider.aliases(provider) ++ OptionsProvider.features(provider))

    assert OptionsProvider.get_canonical_feature_name(provider, "a") == "feature_A"
    assert OptionsProvider.get_canonical_feature_name!(provider, "A") == "feature_A"

    assert OptionsProvider.get_canonical_feature_names(provider, ["a", "feature_B/initial"]) ==
             ["feature_A", "feature_B/initial"]
  end

  test "option helpers work with and without preferences" do
    provider = OptionsProvider.build!(simple_configs())
    prefs = GetOptionsPreferences.new()

    assert OptionsProvider.get_all_options(provider, ["a"])["myConfig"]["rootString2"] ==
             "gets overridden"

    assert OptionsProvider.get_options(provider, "myConfig", ["a"])["myArray"] == [
             "example item 1"
           ]

    prefs =
      GetOptionsPreferences.set_overrides(prefs, %{
        "myConfig" => %{"rootString2" => "override from prefs"}
      })

    assert OptionsProvider.get_all_options(provider, ["a"], prefs)["myConfig"]["rootString2"] ==
             "override from prefs"

    assert OptionsProvider.get_options(provider, "myConfig", ["a"], prefs)["rootString2"] ==
             "override from prefs"
  end

  test "unknown feature names and unknown keys yield errors" do
    provider = OptionsProvider.build!(simple_configs())
    unknown_feature = "does not exist"
    unknown_feature_error = ~s(Feature name "#{unknown_feature}" is not a known feature.)

    assert {:error, ^unknown_feature_error} =
             OptionsProvider.get_canonical_feature_name(provider, unknown_feature)

    assert_raise ArgumentError, unknown_feature_error, fn ->
      OptionsProvider.get_canonical_feature_name!(provider, unknown_feature)
    end

    assert {:error, ^unknown_feature_error} =
             OptionsProvider.get_canonical_feature_names(provider, [unknown_feature])

    assert {:error, ^unknown_feature_error} =
             OptionsProvider.get_all_options(provider, [unknown_feature])

    key = "unknown_key"

    expected_error =
      ~s(Error getting options with features ["feature_A"]: configuration property "#{key}" not found)

    assert {:error, ^expected_error} =
             OptionsProvider.get_options(provider, key, ["feature_A"])

    assert {:error, ^unknown_feature_error} =
             OptionsProvider.get_filtered_feature_names(provider, ["feature_A", unknown_feature])
  end

  test "feature name filtering and mapping honor preferences" do
    provider = OptionsProvider.build!(simple_configs())
    conditions_provider = OptionsProvider.build!(conditions_configs())

    assert OptionsProvider.get_filtered_feature_names(provider, ["a", "feature_B/initial"]) ==
             ["feature_A", "feature_B/initial"]

    assert OptionsProvider.map_feature_names(provider, ["a"]) == ["feature_A"]

    skip_conversion_prefs =
      GetOptionsPreferences.new()
      |> GetOptionsPreferences.set_skip_feature_name_conversion(true)

    assert OptionsProvider.map_feature_names(provider, ["a"], skip_conversion_prefs) == ["a"]

    condition_prefs = constraints_preferences(%{"country" => "US"})

    assert OptionsProvider.get_filtered_feature_names(
             conditions_provider,
             OptionsProvider.features(conditions_provider),
             condition_prefs
           ) == ["B"]

    assert Enum.any?(
             OptionsProvider.features(conditions_provider),
             &OptionsProvider.has_conditions?(conditions_provider, &1)
           )
  end
end
