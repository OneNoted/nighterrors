#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

for pkgdir in \
  "${repo_root}/packaging/aur/nighterrors-git" \
  "${repo_root}/packaging/aur/nighterrors-bin"
do
  (
    cd "${pkgdir}"
    makepkg --printsrcinfo > .SRCINFO
  )
done
