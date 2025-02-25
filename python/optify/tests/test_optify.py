import os
import json
from pathlib import Path

from optify import OptionsProviderBuilder

def test_empty_build():
    builder = OptionsProviderBuilder()
    provider = builder.build()
    assert provider is not None

def run_suite(suite_path: str) -> None:
    provider = OptionsProviderBuilder() \
        .add_directory(os.path.join(suite_path, 'configs')) \
        .build()
    expectations_path = os.path.join(suite_path, 'expectations')
    for test_case in os.listdir(expectations_path):
        expectation_path = os.path.join(expectations_path, test_case)
        with open(expectation_path, 'r') as f:
            expected_info = json.load(f)
        expected_options = expected_info['options']
        features = expected_info['features']
        for key, expected_value in expected_options.items():
            actual_json = provider.get_options_json(key, features)
            actual_options = json.loads(actual_json)
            assert actual_options == expected_value, (
                f"Options for key '{key}' with features {features} do not match "
                f"for test suite at {expectation_path}"
            )

def test_suites():
    test_suites_dir = (Path(__file__) / '../../../../tests/test_suites').resolve()
    for suite in os.listdir(test_suites_dir):
        suite_path = os.path.join(test_suites_dir, suite)
        if os.path.isdir(suite_path):
            run_suite(suite_path)
