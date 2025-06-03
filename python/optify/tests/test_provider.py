from pathlib import Path

from optify import OptionsProviderBuilder


test_suites_dir = (Path(__file__) / '../../../../tests/test_suites').resolve()
builder = OptionsProviderBuilder()
builder.add_directory(str(test_suites_dir / 'simple/configs'))
PROVIDER = builder.build()

def test_features():
    features = PROVIDER.features()
    features.sort()
    assert features == ['A_with_comments', 'feature_A', 'feature_B/initial']

    try:
        PROVIDER.get_options_json('key', ['A'])
        assert False, "Should have raised an error"
    except BaseException as e:
        assert str(e) == "key and feature names should be valid: \"configuration property \\\"key\\\" not found\""


def test_canonical_feature_name():
    assert PROVIDER.get_canonical_feature_name('feaTure_A') == 'feature_A'
    assert PROVIDER.get_canonical_feature_name('feature_B/initial') == 'feature_B/initial'
    assert PROVIDER.get_canonical_feature_name('A_with_comments') == 'A_with_comments'


def test_canonical_feature_names():
    assert PROVIDER.get_canonical_feature_names(['feature_A']) == ['feature_A']
    assert PROVIDER.get_canonical_feature_names(['feature_B/initial']) == ['feature_B/initial']
    assert PROVIDER.get_canonical_feature_names(['A_with_COmments']) == ['A_with_comments']

    assert PROVIDER.get_canonical_feature_names(['A', 'B']) == ['feature_A', 'feature_B/initial']