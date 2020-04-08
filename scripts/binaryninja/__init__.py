from binaryninja.plugin import PluginCommand
from .ief import ief_find_export_bg, ief_find_import_bg, ief_find_library_bg

PluginCommand.register(
    'Import Export Find\\Find binaries that import library named',
    'find binaries that import a library named',
    ief_find_library_bg
)

PluginCommand.register_for_function(
    'Import Export Find\\Find binaries that export current function',
    'find binaries that export the current function',
    ief_find_export_bg
)

PluginCommand.register_for_function(
    'Import Export Find\\Find binaries that import current function',
    'find binaries that import the current function',
    ief_find_import_bg
)
