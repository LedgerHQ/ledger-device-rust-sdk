#!/usr/bin/env bash
# Fail if the ledger_secure_sdk_sys package lacks the target specs or
# link_wrap.sh: `cargo ledger setup` installs them from the published crate.
# Run from the repository root.
set -euo pipefail

cd "$(dirname "$0")/../../ledger_secure_sdk_sys"

# --allow-dirty: listing only, nothing is packaged or published.
files=$(cargo package --list --allow-dirty)
missing=0
for f in devices/nanox/nanox.json devices/nanosplus/nanosplus.json \
         devices/stax/stax.json devices/flex/flex.json \
         devices/apex_p/apex_p.json link_wrap.sh; do
  if ! grep -qxF "$f" <<< "$files"; then
    echo "[ERROR] $f is missing from the ledger_secure_sdk_sys package."
    missing=1
  fi
done
if [ "$missing" -ne 0 ]; then
  echo "[ERROR] Check include/exclude in ledger_secure_sdk_sys/Cargo.toml. Aborting."
  exit 1
fi
echo "[INFO] Target specs and link_wrap.sh are packaged."
