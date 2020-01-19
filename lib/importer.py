import sys
import pyrite

from importlib.abc import MetaPathFinder, SourceLoader
from importlib.machinery import ModuleSpec

class EngineLoader(SourceLoader):
    def __init__(self, fullname):
        self.fullname = fullname

    def get_filename(self, fullname):
        return f"{fullname}.py"

    def get_data(self, filename):
        return pyrite.read_resource(filename)

class EngineMetaFinder(MetaPathFinder):
    def find_spec(self, fullname, path, target=None):
        if len(pyrite.read_resource(f"{fullname}.py")) > 0:
            return ModuleSpec(fullname, EngineLoader(fullname))

# https://docs.python.org/3/library/importlib.html#importing-a-source-file-directly
# currently won't find sub modules, the bottom of the page above shows how python decides to
# import things.

sys.meta_path.insert(0, EngineMetaFinder())

