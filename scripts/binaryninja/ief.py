from os.path import dirname, normpath
from subprocess import run
from binaryninja.binaryview import BinaryViewType
from binaryninja.interaction import DirectoryNameField, TextLineField, \
    get_directory_name_input, get_form_input, show_markdown_report
from binaryninja.plugin import BackgroundTaskThread


def ief_find_export(binview, func):
    """ run `ief <path> -e <symbol>` """
    directory = get_directory_name_input(
        "Directory to search", dirname(binview.file.filename))
    # Assumes `~/.cargo/bin` already exists in PATH
    result = run(["ief", directory.decode('utf-8'), "-e",
                  func.name], capture_output=True)
    directory = directory.decode('utf-8')
    markdown = f"# Results - binaries in '[{directory}]({directory})'" \
               f" with exported symbol '_{func.name}_'`\n\n"
    for line in result.stdout.split(b"\n")[1: -1]:
        npath = normpath(line.decode('utf-8'))
        markdown += f"* [{npath}]({npath})\n"
    show_markdown_report(f"I[E]F - {func.name}",
                         markdown, result.stdout.decode('utf-8'))


def ief_find_import(binview, func):
    """ run `ief <path> -i <symbol>` """
    directory = get_directory_name_input(
        "Directory to search", dirname(binview.file.filename))
    # Assumes `~/.cargo/bin` already exists in PATH
    result = run(["ief", directory.decode('utf-8'), "-i",
                  func.name], capture_output=True)
    directory = directory.decode('utf-8')
    markdown = f"# Results - binaries in '[{directory}]({directory})'" \
               f" with imported symbol '_{func.name}_'`\n\n"
    for line in result.stdout.split(b"\n")[1:-1]:
        npath = normpath(line.decode('utf-8'))
        markdown += f"* [{npath}]({npath})\n"
    show_markdown_report(f"[I]EF - {func.name}",
                         markdown, result.stdout.decode('utf-8'))


def ief_find_library(binview):
    """ run `ief <path> -l <partial library name>` """
    name = TextLineField("Library name (can be partial)")
    directory = DirectoryNameField(
        "Directory to search", dirname(binview.file.filename))
    get_form_input([name, None, directory],
                   "Find binaries that import library")
    if not directory.result:
        directory.result = dirname(binview.file.filename)
    if name.result is None:
        return
    # Assumes `~/.cargo/bin` already exists in PATH
    result = run(["ief", directory.result, "-l",
                  name.result], capture_output=True)
    directory = directory.result
    markdown = f"# Results - binaries in '[{directory}]({directory})'" \
        f"with imported library name containing '_{name.result}_'\n\n"
    for line in result.stdout.split(b"\n")[1:-1]:
        npath = normpath(line.decode('utf-8'))
        markdown += f"* [{npath}]({npath})\n"
    show_markdown_report(f"IEF - {name.result}",
                         markdown, result.stdout.decode('utf-8'))


class IEFExportInBackground(BackgroundTaskThread):
    def __init__(self, binview, func, msg):
        BackgroundTaskThread.__init__(self, msg, True)
        self.binview = binview
        self.func = func

    def run(self):
        ief_find_export(self.binview, self.func)


def ief_find_export_bg(binview, func):
    background_task = IEFExportInBackground(
        binview, func, "Running ief in background")
    background_task.start()
    background_task.finish()


class IEFImportInBackground(BackgroundTaskThread):
    def __init__(self, binview, func, msg):
        BackgroundTaskThread.__init__(self, msg, True)
        self.binview = binview
        self.func = func

    def run(self):
        ief_find_import(self.binview, self.func)


def ief_find_import_bg(binview, func):
    background_task = IEFImportInBackground(
        binview, func, "Running ief in background")
    background_task.start()
    background_task.finish()


class IEFLibraryInBackground(BackgroundTaskThread):
    def __init__(self, binview, msg):
        BackgroundTaskThread.__init__(self, msg, True)
        self.binview = binview

    def run(self):
        ief_find_library(self.binview)


def ief_find_library_bg(binview):
    background_task = IEFLibraryInBackground(
        binview, "Running ief in background")
    background_task.start()
    background_task.finish()
