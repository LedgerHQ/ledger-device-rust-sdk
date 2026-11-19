#!/usr/bin/env bash
# Decides whether the pinned nightly is due for a bump (NIGHTLY.md §4) and
# which nightly to bump to.
#
# Policy: bump every second Rust stable release (~12 weeks), to the nightly the
# current stable release branched from, i.e. the last nightly that still
# reports the stable's version (`1.N.0-nightly`).
#
# Usage: check_nightly_bump.sh [path/to/rust-toolchain.toml]
# Prints key=value lines (pin, pin_minor, stable_minor, recommended, due),
# suitable for appending to $GITHUB_OUTPUT.
set -euo pipefail

toolchain_file=${1:-rust-toolchain.toml}
dist=https://static.rust-lang.org/dist
releases_per_bump=2
tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT

# Minor version of the rustc in a channel manifest:
# `version = "1.93.0-nightly (b33119ffd 2025-12-04)"` -> 93
manifest_minor() {
  awk '/^\[pkg\.rust\]$/ { f = 1; next } f && /^version = / { print; exit }' "$1" |
    sed -E 's/^version = "1\.([0-9]+)\..*/\1/'
}

# Downloads a manifest into $tmp; fails if it does not exist (a nightly can be
# missing when its build failed).
fetch() { curl -fsSL --retry 3 -o "$tmp" "$1"; }

pin=$(sed -nE 's/^channel *= *"nightly-([0-9]{4}-[0-9]{2}-[0-9]{2})".*/\1/p' "$toolchain_file")
if [[ -z "$pin" ]]; then
  echo "No date-pinned nightly channel in $toolchain_file" >&2
  exit 1
fi
fetch "$dist/$pin/channel-rust-nightly.toml"
pin_minor=$(manifest_minor "$tmp")

fetch "$dist/channel-rust-stable.toml"
stable_minor=$(manifest_minor "$tmp")
# Anchor on the 1.N.0 release: the stable channel's date moves with patch
# releases (1.N.1, ...) while the branch point does not.
fetch "$dist/channel-rust-1.$stable_minor.0.toml"
stable_date=$(sed -nE 's/^date = "([0-9-]+)"$/\1/p' "$tmp" | head -n1)

# The stable branched to beta about six weeks before its release. Walk back
# from a date that is safely past the branch point; the first nightly still
# reporting 1.<stable_minor> is the branch point.
recommended=""
start=$(date -u -d "$stable_date - 28 days" +%F)
for i in $(seq 0 35); do
  d=$(date -u -d "$start - $i days" +%F)
  fetch "$dist/$d/channel-rust-nightly.toml" 2>/dev/null || continue
  m=$(manifest_minor "$tmp")
  if [[ "$m" == "$stable_minor" ]]; then
    recommended="nightly-$d"
    break
  fi
  if (( m < stable_minor )); then
    break
  fi
done
if [[ -z "$recommended" ]]; then
  echo "Could not find the nightly that 1.$stable_minor branched from" >&2
  exit 1
fi

due=false
if (( stable_minor - pin_minor >= releases_per_bump )); then
  due=true
fi

echo "pin=nightly-$pin"
echo "pin_minor=$pin_minor"
echo "stable_minor=$stable_minor"
echo "recommended=$recommended"
echo "due=$due"
