"""Every Pylint message, with its checker, and whether it is on by default (W-144). Run: python3 pylint_rules.py out.json"""
import json, sys, pkgutil
import pylint, pylint.extensions
from pylint.lint import PyLinter
from pylint import checkers
linter = PyLinter()
checkers.initialize(linter)
core = {m.msgid for m in linter.msgs_store.messages}
exts = sorted(f'pylint.extensions.{m.name}' for m in pkgutil.iter_modules(pylint.extensions.__path__) if not m.name.startswith('_'))
linter.load_plugin_modules(exts)
out = []
for m in sorted(linter.msgs_store.messages, key=lambda m: m.msgid):
    out.append(dict(tool='pylint', version=pylint.__version__, id=m.msgid, symbol=m.symbol, checker=m.checker_name,
                    category={'C': 'convention', 'R': 'refactor', 'W': 'warning', 'E': 'error', 'F': 'fatal', 'I': 'info'}[m.msgid[0]],
                    extension=m.msgid not in core, old_names=[o[1] for o in m.old_names]))
json.dump(out, open(sys.argv[1], 'w'), indent=0, sort_keys=True)
import collections; print(len(out), collections.Counter(o['category'] for o in out), 'extensions', sum(o['extension'] for o in out))
