#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <version> <sha256>" >&2
  exit 1
fi

version="$1"
sha256="$2"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
pkgbuild="${repo_root}/packaging/aur/nighterrors-bin/PKGBUILD"

python - "$pkgbuild" "$version" "$sha256" <<'PY'
from pathlib import Path
import re
import sys

path = Path(sys.argv[1])
version = sys.argv[2]
sha256 = sys.argv[3]
text = path.read_text()
text = re.sub(r"^pkgver=.*$", f"pkgver={version}", text, flags=re.MULTILINE)
text = re.sub(
    r"^sha256sums=\('[^']*'\)$",
    f"sha256sums=('{sha256}')",
    text,
    flags=re.MULTILINE,
)
path.write_text(text)
PY

"${repo_root}/scripts/update-aur-srcinfo.sh"
