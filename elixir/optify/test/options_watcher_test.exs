defmodule Optify.OptionsWatcherTest do
  use ExUnit.Case, async: true

  import Optify.TestHelpers

  alias Optify.GetOptionsPreferences
  alias Optify.OptionsWatcher

  test "build APIs return watchers and schema variants work" do
    assert {:ok, %OptionsWatcher{} = watcher} = OptionsWatcher.build(simple_configs())
    assert %OptionsWatcher{} = OptionsWatcher.build!(simple_configs())

    assert {:ok, %OptionsWatcher{} = schema_watcher} =
             OptionsWatcher.build_with_schema(simple_configs(), schema_path())

    assert %OptionsWatcher{} = OptionsWatcher.build_with_schema!(simple_configs(), schema_path())

    assert {:ok, %OptionsWatcher{} = from_dirs_watcher} =
             OptionsWatcher.build_from_directories([simple_configs()])

    assert %OptionsWatcher{} = OptionsWatcher.build_from_directories!([simple_configs()])

    assert {:ok, %OptionsWatcher{} = from_dirs_schema_watcher} =
             OptionsWatcher.build_from_directories_with_schema([simple_configs()], schema_path())

    assert %OptionsWatcher{} =
             OptionsWatcher.build_from_directories_with_schema!([simple_configs()], schema_path())

    assert Enum.sort(OptionsWatcher.features(watcher)) == [
             "A_with_comments",
             "feature_A",
             "feature_B/initial"
           ]

    assert Enum.sort(OptionsWatcher.features(schema_watcher)) ==
             OptionsWatcher.features(watcher) |> Enum.sort()

    assert Enum.sort(OptionsWatcher.features(from_dirs_watcher)) ==
             OptionsWatcher.features(watcher) |> Enum.sort()

    assert Enum.sort(OptionsWatcher.features(from_dirs_schema_watcher)) ==
             OptionsWatcher.features(watcher) |> Enum.sort()
  end

  test "build returns errors and bang variants raise on invalid directories" do
    assert {:error, _} = OptionsWatcher.build(missing_directory())
    assert {:error, _} = OptionsWatcher.build_with_schema(missing_directory(), schema_path())
    assert {:error, _} = OptionsWatcher.build_from_directories([missing_directory()])

    assert {:error, _} =
             OptionsWatcher.build_from_directories_with_schema(
               [missing_directory()],
               schema_path()
             )

    assert_raise ArgumentError, fn -> OptionsWatcher.build!(missing_directory()) end

    assert_raise ArgumentError, fn ->
      OptionsWatcher.build_with_schema!(missing_directory(), schema_path())
    end

    assert_raise ArgumentError, fn ->
      OptionsWatcher.build_from_directories!([missing_directory()])
    end

    assert_raise ArgumentError, fn ->
      OptionsWatcher.build_from_directories_with_schema!([missing_directory()], schema_path())
    end
  end

  test "build wraps native references and get_options passes through native errors" do
    native_watcher_ref = Optify.Native.watcher_build(simple_configs())

    assert is_reference(native_watcher_ref)

    assert {:ok, %OptionsWatcher{ref: wrapper_watcher_ref}} =
             OptionsWatcher.build(simple_configs())

    assert is_reference(wrapper_watcher_ref)

    assert {:error,
            ~s(Error getting options with features ["feature_A"]: configuration property "unknown_key" not found)} =
             Optify.Native.watcher_get_options(
               wrapper_watcher_ref,
               "unknown_key",
               ["feature_A"],
               GetOptionsPreferences.new().ref
             )

    assert {:error,
            ~s(Error getting options with features ["feature_A"]: configuration property "unknown_key" not found)} =
             OptionsWatcher.get_options(
               %OptionsWatcher{ref: wrapper_watcher_ref},
               "unknown_key",
               ["feature_A"]
             )
  end

  test "query helpers expose aliases, features, and canonical feature names" do
    watcher = OptionsWatcher.build!(simple_configs())

    assert Enum.sort(OptionsWatcher.aliases(watcher)) == ["a", "b"]

    assert Enum.sort(OptionsWatcher.features(watcher)) == [
             "A_with_comments",
             "feature_A",
             "feature_B/initial"
           ]

    assert Enum.sort(OptionsWatcher.features_and_aliases(watcher)) ==
             Enum.sort(OptionsWatcher.aliases(watcher) ++ OptionsWatcher.features(watcher))

    assert OptionsWatcher.get_canonical_feature_name(watcher, "a") == "feature_A"
    assert OptionsWatcher.get_canonical_feature_name!(watcher, "A") == "feature_A"

    assert OptionsWatcher.get_canonical_feature_names(watcher, ["a", "feature_B/initial"]) ==
             ["feature_A", "feature_B/initial"]
  end

  test "option helpers work with and without preferences and expose timestamps" do
    watcher = OptionsWatcher.build!(simple_configs())
    prefs = GetOptionsPreferences.new()

    assert OptionsWatcher.get_all_options(watcher, ["a"])["myConfig"]["rootString2"] ==
             "gets overridden"

    assert OptionsWatcher.get_options(watcher, "myConfig", ["a"])["myArray"] == ["example item 1"]

    prefs =
      GetOptionsPreferences.set_overrides_json(
        prefs,
        ~s({"myConfig":{"rootString2":"override from json"}})
      )

    assert OptionsWatcher.get_all_options(watcher, ["a"], prefs)["myConfig"]["rootString2"] ==
             "override from json"

    assert OptionsWatcher.get_options(watcher, "myConfig", ["a"], prefs)["rootString2"] ==
             "override from json"

    assert is_integer(OptionsWatcher.last_modified(watcher))
    assert OptionsWatcher.last_modified(watcher) > 0
  end

  test "feature name filtering and mapping honor preferences" do
    watcher = OptionsWatcher.build!(simple_configs())
    conditions_watcher = OptionsWatcher.build!(conditions_configs())

    assert OptionsWatcher.get_filtered_feature_names(watcher, ["a", "feature_B/initial"]) ==
             ["feature_A", "feature_B/initial"]

    assert OptionsWatcher.map_feature_names(watcher, ["a"]) == ["feature_A"]

    skip_conversion_prefs =
      GetOptionsPreferences.new()
      |> GetOptionsPreferences.set_skip_feature_name_conversion(true)

    assert OptionsWatcher.map_feature_names(watcher, ["a"], skip_conversion_prefs) == ["a"]

    condition_prefs = constraints_preferences(%{"country" => "US"})

    assert OptionsWatcher.get_filtered_feature_names(
             conditions_watcher,
             OptionsWatcher.features(conditions_watcher),
             condition_prefs
           ) == ["B"]

    assert Enum.any?(
             OptionsWatcher.features(conditions_watcher),
             &OptionsWatcher.has_conditions?(conditions_watcher, &1)
           )
  end
end
