import os
import json
from pathlib import Path

from optify import GetOptionsPreferences, OptionsProvider, OptionsProviderBuilder

def test_bad_directory():
    builder = OptionsProviderBuilder()
    directory = 'bad_directory'
    try:
        builder.add_directory(directory)
        assert False, "Should have raised an error"
    except Exception as e:
        assert str(e) == f"Error adding directory: \"{directory}\" is not a directory"
