# Syntax that only exists on Python 3.10+.
#
# This fixture exists to answer one question: when py2pyd targets an *older*
# interpreter (say 3.9) but the source uses newer syntax, does the build still
# work, and does the module still run on the older interpreter?
#
# Cython has its own parser, so the answer tells us whether pinning the default
# Limited API target low would break users who write modern Python.


def classify(value):
    match value:
        case 0:
            return "zero"
        case [x, y]:
            return "pair {} {}".format(x, y)
        case str() as s:
            return "str " + s
        case _:
            return "other"


def union_join(value):
    # PEP 604 union annotation, quoted so the target interpreter never parses it.
    result: "str | None" = value
    return result


MODULE_TAG = "syntax310"
