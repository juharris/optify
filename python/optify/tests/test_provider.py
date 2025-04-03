import os
import json
from pathlib import Path
import sys

from optify import OptionsProviderBuilder

def test_features():
    test_suites_dir = (Path(__file__) / '../../../../tests/test_suites').resolve()
    builder = OptionsProviderBuilder()
    builder.add_directory(str(test_suites_dir / 'simple/configs'))
    provider = builder.build()
    features = provider.features()
    features.sort()
    assert features == ['A_with_comments', 'feature_A', 'feature_B/initial']

    try:
        provider.get_options_json('key', ['A'])
        assert False, "Should have raised an error"
    except:
        # Can't get pyo3_runtime.PanicException because can't import it and catching `Exception` doesn't work either.
        e = sys.exc_info()[1]
        assert str(e) == "Failed to get options: \"configuration property \\\"key\\\" not found\""
