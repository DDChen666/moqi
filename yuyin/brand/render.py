"""Render Yuyin's app icon and tray icons from yuyin/brand/render.html.

Usage (from the app/ directory):
    python3 yuyin/brand/render.py

Writes:
    yuyin/brand/app-icon-1024.png      source for `bun tauri icon`
    src-tauri/resources/tray_*.png     menu bar icons
    src-tauri/resources/{handy,recording,transcribing,handy_warning}.png
Then run `bun tauri icon yuyin/brand/app-icon-1024.png` to regenerate src-tauri/icons/.
"""

import base64
import json
import re
import subprocess
import sys
from pathlib import Path

CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
HERE = Path(__file__).resolve().parent
APP = HERE.parent.parent


def main():
    dom = subprocess.run(
        [CHROME, "--headless=new", "--disable-gpu", "--virtual-time-budget=5000",
         "--dump-dom", (HERE / "render.html").as_uri()],
        capture_output=True, text=True, check=True,
    ).stdout
    match = re.search(r'<pre id="out">(\{.*?)</pre>', dom, re.S)
    if not match or not match.group(1).strip():
        sys.exit("render.html produced no output")
    images = json.loads(match.group(1).replace("&quot;", '"').replace("&amp;", "&"))
    for name, url in images.items():
        data = base64.b64decode(url.split(",", 1)[1])
        dest = HERE / name if name.startswith("app-icon") else APP / "src-tauri/resources" / name
        dest.write_bytes(data)
        print(f"{dest.relative_to(APP)}  {len(data)} bytes")


if __name__ == "__main__":
    main()
