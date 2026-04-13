#!/usr/bin/env bash

set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <version> [target-triple]" >&2
  exit 1
fi

version="$1"
target_triple="${2:-x86_64-unknown-linux-gnu}"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dist_dir="${repo_root}/dist"
staging_root="${dist_dir}/staging"
asset_name="nighterrors-${version}-${target_triple}"
staging_dir="${staging_root}/${asset_name}"
archive_path="${dist_dir}/${asset_name}.tar.xz"

binary_path="${repo_root}/target/${target_triple}/release/nighterrors"
if [[ ! -x "${binary_path}" ]]; then
  binary_path="${repo_root}/target/release/nighterrors"
fi

if [[ ! -x "${binary_path}" ]]; then
  echo "missing built binary: ${binary_path}" >&2
  exit 1
fi

rm -rf "${staging_dir}" "${archive_path}"
mkdir -p "${staging_dir}"

install -Dm755 "${binary_path}" "${staging_dir}/nighterrors"
install -Dm644 "${repo_root}/README.md" "${staging_dir}/README.md"
install -Dm644 "${repo_root}/LICENSE" "${staging_dir}/LICENSE"

mkdir -p "${dist_dir}"
tar \
  --sort=name \
  --owner=0 \
  --group=0 \
  --numeric-owner \
  --mtime='UTC 1970-01-01' \
  -C "${staging_root}" \
  -cJf "${archive_path}" \
  "${asset_name}"

(
  cd "${dist_dir}"
  sha256sum "${asset_name}.tar.xz" > SHA256SUMS
)

echo "Built ${archive_path}"
