from pathlib import Path
from typing import List

from pydantic import BaseModel

from optify import OptionsProvider, OptionsWatcher


test_suites_dir = (Path(__file__) / '../../../../tests/test_suites').resolve()
simple_configs_dir = str(test_suites_dir / 'simple/configs')

PROVIDERS = [
    OptionsProvider.build(simple_configs_dir),
    OptionsWatcher.build(simple_configs_dir),
]


class Deeper(BaseModel):
    wtv: int
    list: List[int]
    new: str


class MyObject(BaseModel):
    one: int
    two: int
    three: int
    string: str
    deeper: Deeper


class MyConfig(BaseModel):
    myArray: List[str]
    myObject: MyObject
    rootString: str
    rootString2: str


def test_get_options_pydantic():
    for provider in PROVIDERS:
        options_json = provider.get_options_json(
            'myConfig', ['feature_A', 'feature_B/initial'])
        config = MyConfig.model_validate_json(options_json)

        assert isinstance(config, MyConfig)
        assert config.rootString == 'root string same'
        assert config.rootString2 == 'override'
        assert config.myArray == ['different item 1', 'item 2']

        assert isinstance(config.myObject, MyObject)
        assert config.myObject.one == 1
        assert config.myObject.two == 22
        assert config.myObject.three == 3
        assert config.myObject.string == 'string'

        assert isinstance(config.myObject.deeper, Deeper)
        assert config.myObject.deeper.wtv == 3333
        assert config.myObject.deeper.list == [55]
        assert config.myObject.deeper.new == 'new value'
