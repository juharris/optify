defmodule Optify.OptionsProviderBuilderTest do
  use ExUnit.Case, async: true

  import Optify.TestHelpers

  alias Optify.OptionsProvider
  alias Optify.OptionsProviderBuilder

  test "new, add_directory, and build return provider structs" do
    builder = OptionsProviderBuilder.new()
    assert {:ok, ^builder} = OptionsProviderBuilder.add_directory(builder, simple_configs())
    assert {:ok, %OptionsProvider{} = provider} = OptionsProviderBuilder.build(builder)
    assert Enum.sort(OptionsProvider.aliases(provider)) == ["a", "b"]
  end

  test "bang builder helpers return updated builder and provider" do
    builder = OptionsProviderBuilder.new()
    builder = OptionsProviderBuilder.add_directory!(builder, simple_configs())
    provider = OptionsProviderBuilder.build!(builder)
    assert %OptionsProvider{} = provider

    assert Enum.sort(OptionsProvider.features(provider)) == [
             "A_with_comments",
             "feature_A",
             "feature_B/initial"
           ]
  end

  test "builder helpers surface invalid directory errors" do
    builder = OptionsProviderBuilder.new()
    assert {:error, _} = OptionsProviderBuilder.add_directory(builder, missing_directory())

    assert_raise ArgumentError, fn ->
      OptionsProviderBuilder.add_directory!(builder, missing_directory())
    end
  end
end
