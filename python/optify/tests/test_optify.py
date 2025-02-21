import optify

from optify import OptionsProviderBuilder

def test_module():
    print(optify)
    print(optify.__all__)

def test_suites():
    # TODO Test all suites.
    builder = OptionsProviderBuilder()
    builder.add_directory('../../tests/test_suites/simple/configs')
    provider = builder.build()
    j = provider.get_options_json('myConfig', ['A'])
    assert j == "TODO"


# TODO Remove and use pytest.
if __name__ == '__main__':
    test_module()