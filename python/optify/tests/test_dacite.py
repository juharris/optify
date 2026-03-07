import json
from dataclasses import dataclass
from pathlib import Path
from typing import List

import dacite

from optify import OptionsProvider, OptionsWatcher


test_suites_dir = (Path(__file__) / '../../../../tests/test_suites').resolve()
simple_configs_dir = str(test_suites_dir / 'simple/configs')

PROVIDERS = [
    OptionsProvider.build(simple_configs_dir),
    OptionsWatcher.build(simple_configs_dir),
]


@dataclass
class Deeper:
    wtv: int
    list: List[int]
    new: str


@dataclass
class MyObject:
    one: int
    two: int
    three: int
    string: str
    deeper: Deeper


@dataclass
class MyConfig:
    myArray: List[str]
    myObject: MyObject
    rootString: str
    rootString2: str


def test_get_options_dacite():
    for provider in PROVIDERS:
        options_json = provider.get_options_json(
            'myConfig', ['feature_A', 'feature_B/initial'])
        config = dacite.from_dict(data_class=MyConfig,
                                  data=json.loads(options_json))

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
