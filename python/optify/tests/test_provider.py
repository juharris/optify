from pathlib import Path

from optify import OptionsProvider, OptionsProviderBuilder


test_suites_dir = (Path(__file__) / '../../../../tests/test_suites').resolve()
simple_configs_dir = str(test_suites_dir / 'simple/configs')
builder = OptionsProviderBuilder()
builder.add_directory(simple_configs_dir)
PROVIDER = builder.build()

def test_features():
    features = PROVIDER.features()
    features.sort()
    assert features == ['A_with_comments', 'feature_A', 'feature_B/initial']

    try:
        PROVIDER.get_options_json('key', ['A'])
        assert False, "Should have raised an error"
    except Exception as e:
        assert str(e) == "configuration property \"key\" not found"


def test_canonical_feature_name():
    assert PROVIDER.get_canonical_feature_name('feaTure_A') == 'feature_A'
    assert PROVIDER.get_canonical_feature_name('feature_B/initial') == 'feature_B/initial'
    assert PROVIDER.get_canonical_feature_name('A_with_comments') == 'A_with_comments'


def test_canonical_feature_names():
    assert PROVIDER.get_canonical_feature_names(['feature_A']) == ['feature_A']
    assert PROVIDER.get_canonical_feature_names(['feature_B/initial']) == ['feature_B/initial']
    assert PROVIDER.get_canonical_feature_names(['A_with_COmments']) == ['A_with_comments']

    assert PROVIDER.get_canonical_feature_names(['A', 'B']) == ['feature_A', 'feature_B/initial']

def test_build_from_directories():
    provider = OptionsProvider.build_from_directories([simple_configs_dir])
    assert provider is not None
    features = provider.features()
    features.sort()
    assert features == ['A_with_comments', 'feature_A', 'feature_B/initial']

    assert provider.get_options_json('myConfig', ['feature_A', 'feature_B/initial'])