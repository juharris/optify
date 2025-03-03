import os
import json
from pathlib import Path

from optify import OptionsProviderBuilder

def test_features():
    test_suites_dir = (Path(__file__) / '../../../../tests/test_suites').resolve()
    builder = OptionsProviderBuilder()
    builder.add_directory(str(test_suites_dir / 'simple/configs'))
    provider = builder.build()
    features = provider.features()
    features.sort()
    assert features == ['A_with_comments', 'feature_A', 'feature_B/initial']
