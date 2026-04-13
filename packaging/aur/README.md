# AUR packaging

This repository ships the source files for two separate AUR packages:

- `nighterrors-git`: builds the latest repository state from Git
- `nighterrors-bin`: installs the prebuilt release archive published from GitHub Releases

## Layout

- `packaging/aur/nighterrors-git`: AUR repo contents for the VCS package
- `packaging/aur/nighterrors-bin`: AUR repo contents for the binary package

## Release asset contract

`nighterrors-bin` expects this exact GitHub release asset layout:

- tag: `v<version>`
- asset: `nighterrors-<version>-x86_64-unknown-linux-gnu.tar.xz`
- archive contents:
  - `nighterrors`
  - `README.md`
  - `LICENSE`

The helper script [scripts/build-release-archive.sh](/home/notes/Projects/nighterrors/scripts/build-release-archive.sh)
builds that archive, and the GitHub Actions workflow
[.github/workflows/release.yml](/home/notes/Projects/nighterrors/.github/workflows/release.yml)
uploads it automatically when a `v*` tag is pushed.

## Publish flow

1. Push the repository changes and create a release tag such as `v0.1.0`.
2. Let the release workflow upload the binary archive and `SHA256SUMS`.
3. Update the binary AUR package metadata:
   `scripts/update-aur-bin-package.sh 0.1.0 <sha256>`
4. Refresh both `.SRCINFO` files:
   `scripts/update-aur-srcinfo.sh`
5. Copy each package directory into its matching AUR Git repo and publish:
   - `packaging/aur/nighterrors-git`
   - `packaging/aur/nighterrors-bin`
