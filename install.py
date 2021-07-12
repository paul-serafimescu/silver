#!/usr/bin/env python3

"""obviously this is terrible at the moment
i'll fix it later lol
"""

import sys
import os
import subprocess
import shutil

PLATFORM = sys.platform
HOME = os.path.expanduser("~")

if __name__ == '__main__':
  if 'linux' in PLATFORM:
    subprocess.call(["cargo", "install", "--path", "."])
    try:
      os.mkdir(os.path.join(HOME, ".editrc"))
    except OSError as error:
      pass
    shutil.copytree("syntax", os.path.join(HOME, ".editrc/syntax"), dirs_exist_ok=True)
