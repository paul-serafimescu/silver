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
    try:
      subprocess.run(["cargo", "install", "--path", "."]).check_returncode()
    except subprocess.CalledProcessError as error:
      print(error, file=sys.stderr)
    try:
      os.mkdir(os.path.join(HOME, ".editrc"))
    except OSError as error:
      if isinstance(error, FileExistsError):
        pass
      else:
        pass
    shutil.copytree("syntax", os.path.join(HOME, ".editrc/syntax"), dirs_exist_ok=True)
