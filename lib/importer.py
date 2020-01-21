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
        return pyrite.resource_read(filename)

class EngineMetaFinder(MetaPathFinder):
    def find_spec(self, fullname, path, target=None):
        if pyrite.resource_exists(f"{fullname}.py"):
            return ModuleSpec(fullname, EngineLoader(fullname))

sys.meta_path.append(EngineMetaFinder())

