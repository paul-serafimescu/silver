#!/usr/bin/env python3

"""obviously this is terrible at the moment
i'll fix it later lol
"""

import sys
import os
import subprocess

PLATFORM = sys.platform

if __name__ == '__main__':
  if 'linux' in PLATFORM:
    subprocess.call(["cargo", "build", "--release"])
    path = os.path.join(os.path.abspath(os.path.curdir), "target", "release")
    with open(os.path.expanduser("~/.bashrc"), "a") as config:
      config.write(f"export PATH=\"{path}:$PATH\"")
