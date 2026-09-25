# Probes the *semantics* Cython compiled the module with, so a change in the
# effective `language_level` shows up as a value change instead of a warning.
#
# py2pyd's generated setup.py sets no `language_level` directive. Cython's own
# default for that directive is documented as "warn but default to 2", so this
# fixture is the guard: if a future Cython or setuptools starts defaulting to
# Python 2 semantics, `div(7, 2)` returns 3 instead of 3.5.


def div(a, b):
    return a / b


def str_kind():
    return type("x").__name__


def add_str(a, b):
    return a + b


MODULE_TAG = "levelprobe"
