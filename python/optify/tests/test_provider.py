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

    try:
        provider.get_options_json('key', ['A'])
        assert False, "Should have raised an error"
    except BaseException as e:
        assert str(e) == "key and feature names should be valid: \"configuration property \\\"key\\\" not found\""
