defmodule Optify.OptionsWatcherBuilderTest do
  use ExUnit.Case, async: true

  import Optify.TestHelpers

  alias Optify.OptionsWatcher
  alias Optify.OptionsWatcherBuilder

  test "new, add_directory, and build return watcher structs" do
    builder = OptionsWatcherBuilder.new()
    assert {:ok, ^builder} = OptionsWatcherBuilder.add_directory(builder, simple_configs())
    assert {:ok, %OptionsWatcher{} = watcher} = OptionsWatcherBuilder.build(builder)
    assert Enum.sort(OptionsWatcher.aliases(watcher)) == ["a", "b"]
  end

  test "bang builder helpers return updated builder and watcher" do
    builder = OptionsWatcherBuilder.new()
    builder = OptionsWatcherBuilder.add_directory!(builder, simple_configs())
    watcher = OptionsWatcherBuilder.build!(builder)
    assert %OptionsWatcher{} = watcher

    assert Enum.sort(OptionsWatcher.features(watcher)) == [
             "A_with_comments",
             "feature_A",
             "feature_B/initial"
           ]
  end

  test "builder helpers surface invalid directory errors" do
    builder = OptionsWatcherBuilder.new()
    assert {:ok, ^builder} = OptionsWatcherBuilder.add_directory(builder, missing_directory())
    assert {:error, _} = OptionsWatcherBuilder.build(builder)

    builder = OptionsWatcherBuilder.new()
    builder = OptionsWatcherBuilder.add_directory!(builder, missing_directory())

    assert_raise ArgumentError, fn ->
      OptionsWatcherBuilder.build!(builder)
    end
  end
end
