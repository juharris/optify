# Manually maintained interfaces as explained in: https://pyo3.rs/v0.25.1/python-typing-hints.html
# Eventually we should be able to automatically generate this as explained there.

from abc import ABC
from typing import List, Optional, Self


class GetOptionsPreferences:
    def are_configurable_strings_enabled(self) -> bool: ...
    def enable_configurable_strings(self) -> None: ...
    def disable_configurable_strings(self) -> None: ...
    def set_constraints_json(
        self, constraints_json: Optional[str]) -> None: ...


class OptionsProviderBuilderBase(ABC):
    def add_directory(self, directory: str) -> Self: ...


class OptionsProviderBase(ABC):
    @classmethod
    def build(cls, directory: str) -> Self: ...

    @classmethod
    def build_from_directories(
        cls, directories: List[str]) -> Self: ...

    def features(self) -> List[str]: ...

    def get_options_json(self, key: str, feature_names: List[str]) -> str: ...

    def get_options_json_with_preferences(
        self, key: str, feature_names: List[str], preferences: Optional[GetOptionsPreferences]) -> str: ...


class OptionsProvider(OptionsProviderBase):
    pass


class OptionsProviderBuilder(OptionsProviderBuilderBase):
    def build(self) -> OptionsProvider: ...


class OptionsWatcher(OptionsProviderBase):
    pass


class OptionsWatcherBuilder(OptionsProviderBuilderBase):
    def build(self) -> OptionsWatcher: ...
