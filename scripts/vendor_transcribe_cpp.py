"""Put patched copies of transcribe-cpp / transcribe-cpp-sys in src-tauri/vendor/.

Moqi patches the speech engine so Qwen3-ASR can take dictionary words as
recognition context (src-tauri/patches/). Cargo's [patch.crates-io] points at
the vendored copies, so run this once before building, and again whenever the
patch changes (it is cheap when nothing changed):

    python3 scripts/vendor_transcribe_cpp.py

The pristine crates come from the local cargo registry, or from crates.io when
they aren't there yet. src-tauri/vendor/ is not committed.
"""

import hashlib
import io
import os
import shutil
import ssl
import subprocess
import sys
import tarfile
import urllib.request
from pathlib import Path

VERSION = "0.2.4"
CRATES = ["transcribe-cpp-sys", "transcribe-cpp"]
ROOT = Path(__file__).resolve().parent.parent
PATCH = ROOT / "src-tauri/patches/transcribe-cpp-0.2.4-recognition-context.patch"
VENDOR = ROOT / "src-tauri/vendor"
STAMP = VENDOR / ".patched"


def registry_copy(name: str) -> Path | None:
    registry = Path.home() / ".cargo/registry/src"
    for index in sorted(registry.glob("*")) if registry.exists() else []:
        candidate = index / f"{name}-{VERSION}"
        if (candidate / "Cargo.toml").exists():
            return candidate
    return None


def download(name: str, dest: Path) -> None:
    url = f"https://static.crates.io/crates/{name}/{name}-{VERSION}.crate"
    # python.org builds on macOS ship without CA certificates; use the system's.
    cafile = "/etc/ssl/cert.pem" if Path("/etc/ssl/cert.pem").exists() else None
    with urllib.request.urlopen(url, context=ssl.create_default_context(cafile=cafile)) as r:
        data = r.read()
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as tar:
        tar.extractall(dest.parent / "_download", filter="data")
    shutil.move(str(dest.parent / "_download" / f"{name}-{VERSION}"), dest)
    shutil.rmtree(dest.parent / "_download", ignore_errors=True)


def main() -> int:
    digest = hashlib.sha256(PATCH.read_bytes()).hexdigest()
    if STAMP.exists() and STAMP.read_text().strip() == digest and all((VENDOR / c).exists() for c in CRATES):
        print("vendor/ is up to date")
        return 0
    shutil.rmtree(VENDOR, ignore_errors=True)
    VENDOR.mkdir(parents=True)
    for name in CRATES:
        dest = VENDOR / name
        src = registry_copy(name)
        if src:
            shutil.copytree(src, dest)
        else:
            print(f"downloading {name} {VERSION} from crates.io")
            download(name, dest)
        # Registry sources carry a checksum manifest cargo would verify.
        (dest / ".cargo-checksum.json").unlink(missing_ok=True)
    # Inside the app's own repo, git would read the patch's paths relative to
    # the repo root and skip them; stop it from looking above vendor/.
    env = {**os.environ, "GIT_CEILING_DIRECTORIES": str(VENDOR.parent)}
    subprocess.run(["git", "apply", "-p1", "--verbose", str(PATCH)], cwd=VENDOR, env=env, check=True)
    STAMP.write_text(digest + "\n")
    print(f"vendored {', '.join(CRATES)} {VERSION} with {PATCH.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
