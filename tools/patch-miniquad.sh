#!/usr/bin/env bash
# Builds patched/miniquad: the published miniquad with patches/miniquad-0.4.11-aarch64.patch applied.
# Cargo.toml points [patch.crates-io] at that folder, so run this once after cloning (cargo build
# fails until the folder exists) and again whenever the patch or the miniquad version changes.
# See patches/README.md.
#
# The crate comes from cargo's download cache if it is there, otherwise from crates.io (CRATES_URL
# overrides the address). Either way it must match the checksum crates.io publishes for it. (Cargo.lock
# cannot supply it: a crate overridden by [patch] is recorded there without one.)
set -euo pipefail
cd "$(dirname "$0")/.."

version=0.4.11
checksum=dcbaea978d741f3b60ced666b06a2a1e4a2bc81cb8b9dd29f4a40965e1f59e65
patch="patches/miniquad-$version-aarch64.patch"
target=patched/miniquad
stamp="$target/.patch-sha256"

wanted=$(sha256sum "$patch" | cut -d' ' -f1)
if [ -f "$stamp" ] && [ "$(cat "$stamp")" = "$wanted" ]; then
  echo "$target is up to date"
  exit 0
fi

if ! grep -A1 '^name = "miniquad"$' Cargo.lock | grep -q "^version = \"$version\"$"; then
  echo "Cargo.lock does not use miniquad $version; update this script and the patch" >&2
  exit 1
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
crate="$work/miniquad.crate"

cached=$(ls "${CARGO_HOME:-$HOME/.cargo}"/registry/cache/*/miniquad-"$version".crate 2>/dev/null | head -n1 || true)
if [ -n "$cached" ]; then
  cp "$cached" "$crate"
else
  curl -fsSL "${CRATES_URL:-https://static.crates.io/crates/miniquad/miniquad-$version.crate}" -o "$crate"
fi
echo "$checksum  $crate" | sha256sum -c --quiet - || {
  echo "miniquad $version does not match the checksum pinned in this script" >&2
  exit 1
}

tar -xzf "$crate" -C "$work"
rm -rf "$target"
mkdir -p "$(dirname "$target")"
mv "$work/miniquad-$version" "$target"
git apply --directory="$target" "$patch"
echo "$wanted" >"$stamp"
echo "built $target"
