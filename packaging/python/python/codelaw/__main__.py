"""``python -m codelaw``: run the binary with this interpreter's site-packages,
so it finds the codelaw-rules-* packs installed beside it."""

import os
import subprocess
import sys

from codelaw import binary_path, environment


def main() -> int:
    bin_ = binary_path()
    if sys.platform == "win32":
        return subprocess.run([bin_, *sys.argv[1:]], env=environment()).returncode
    os.execve(bin_, [bin_, *sys.argv[1:]], environment())
    return 2  # unreachable


if __name__ == "__main__":
    sys.exit(main())
