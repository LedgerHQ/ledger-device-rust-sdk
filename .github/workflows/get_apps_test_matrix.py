#!/usr/bin/env python3
"""Build the GitHub Actions matrices for the manual "Ragger tests for selected
Rust apps" workflow.

Given an explicit list of LedgerHQ application repositories, this script queries
each app's `ledger_app.toml` manifest (through `ledgered`) and emits two JSON
matrices:

  * ``build_matrix.json``     — one entry per (app, device) pair to build.
  * ``apps_matrix.json``      — one entry per app to run Ragger tests on.
  * ``run_for_devices.json``  — the device list (in Ragger naming) to forward to
                                the Ragger reusable workflow, or the literal
                                ``None`` when no device filter was given (so each
                                app falls back to its full manifest device set).

Inputs:
  * argv[1]            : GitHub token (used to read the manifests via the API).
  * env ``APP_LIST``   : space- and/or comma-separated app specs. Each spec is an
                         app repository name (e.g. ``app-boilerplate-rust``),
                         optionally suffixed with ``@<git-ref>`` to test a
                         specific branch/tag of that app. A leading
                         ``LedgerHQ/`` is tolerated and stripped.
  * env ``DEVICES``    : optional space-/comma-separated device filter
                         (``nanos+``, ``nanox``, ``stax``, ``flex``,
                         ``apex_p``). When empty, every device declared by the
                         app's manifest is used.

Only Rust apps are kept; anything else (non-Rust, missing manifest, unknown
repo) is reported and skipped. The script fails if no testable (app, device)
pair remains, so the workflow surfaces an obviously empty selection instead of
silently doing nothing.
"""

from ledgered.github import GitHubLedgerHQ, NoManifestException
from github.GithubException import GithubException

import json
import os
import re
import sys

# Devices the Rust SDK actually targets. `ledgered` reports devices using their
# `sdk_name` (so Nano S+ is "nanos+"); the deprecated Nano S and the apex_m
# variant are not Rust targets and are dropped.
SUPPORTED_DEVICES = {"nanos+", "nanox", "stax", "flex", "apex_p"}

# Ragger / `reusable_ragger_tests.yml` refer to the Nano S+ as "nanosp".
RAGGER_DEVICE_NAME = {"nanos+": "nanosp"}

# `cargo ledger build` (and the cargo target directory) name the Nano S+ target
# "nanosplus". This doubles as a filesystem-safe artifact suffix.
CARGO_TARGET_NAME = {"nanos+": "nanosplus"}


def parse_app_specs(raw: str):
    """Split the free-form APP_LIST into (app_name, ref-or-None) tuples."""
    specs = []
    for token in re.split(r"[\s,]+", raw.strip()):
        if not token:
            continue
        name, _, ref = token.partition("@")
        # Tolerate an explicit "LedgerHQ/" owner prefix.
        name = name.split("/")[-1]
        specs.append((name, ref or None))
    return specs


def main():
    if len(sys.argv) != 2:
        print("Usage: get_apps_test_matrix.py <github_token>", file=sys.stderr)
        print("       (the app list is read from the APP_LIST env variable)", file=sys.stderr)
        sys.exit(1)

    token = sys.argv[1]
    raw_apps = os.environ.get("APP_LIST", "")
    specs = parse_app_specs(raw_apps)
    if not specs:
        print("ERROR: no application provided in APP_LIST", file=sys.stderr)
        sys.exit(1)

    device_filter = {
        d.strip().lower()
        for d in re.split(r"[\s,]+", os.environ.get("DEVICES", "").strip())
        if d.strip()
    }
    unknown_filter = device_filter - SUPPORTED_DEVICES
    if unknown_filter:
        print(f"ERROR: unknown device(s) in DEVICES filter: {sorted(unknown_filter)}", file=sys.stderr)
        print(f"       supported devices are: {sorted(SUPPORTED_DEVICES)}", file=sys.stderr)
        sys.exit(1)

    gh = GitHubLedgerHQ(token)

    build_matrix = []
    apps_matrix = []

    for name, ref in specs:
        if not name.startswith("app-"):
            print(f"ERROR: '{name}' is not a valid Ledger app repository name "
                  "(it must start with 'app-')", file=sys.stderr)
            sys.exit(1)

        try:
            app = gh.get_app(name)
            if ref is not None:
                # Resolve and validate the requested ref against the repo.
                app.current_branch = ref
            resolved_ref = app.current_branch
            manifest = app.manifest
        except NoManifestException:
            print(f"WARNING: skipping '{name}': no 'ledger_app.toml' manifest found")
            continue
        except GithubException as exc:
            print(f"WARNING: skipping '{name}': cannot access repository ({exc.data if hasattr(exc, 'data') else exc})")
            continue

        if not manifest.app.is_rust:
            print(f"WARNING: skipping '{name}': not a Rust app (sdk='{manifest.app.sdk}')")
            continue

        # `manifest.app.devices` is a set of sdk_name strings (e.g. "nanos+").
        devices = {str(d).lower() for d in manifest.app.devices} & SUPPORTED_DEVICES
        if device_filter:
            devices &= device_filter
        if not devices:
            print(f"WARNING: skipping '{name}': no supported/selected device to build")
            continue

        # Artifact name must be filesystem/GitHub-artifact safe.
        artifact = f"ragger-bins-{re.sub(r'[^A-Za-z0-9._-]', '-', name)}"

        apps_matrix.append({"name": name, "ref": resolved_ref, "artifact": artifact})
        for device in sorted(devices):
            build_matrix.append({
                "app": name,
                "ref": resolved_ref,
                "device": device,
                # cargo target / target-dir name, also used as artifact suffix.
                "target": CARGO_TARGET_NAME.get(device, device),
                "artifact": artifact,
            })

    if not build_matrix:
        print("ERROR: no testable (app, device) pair found for the given input", file=sys.stderr)
        sys.exit(1)

    # When the user pinned a device subset, forward it (in Ragger naming) to the
    # test workflow so it only runs the devices we actually built. Otherwise let
    # each app default to its full manifest device set.
    if device_filter:
        run_for_devices = json.dumps(
            sorted(RAGGER_DEVICE_NAME.get(d, d) for d in device_filter)
        )
    else:
        run_for_devices = "None"

    with open("build_matrix.json", "w") as f:
        f.write(json.dumps(build_matrix))
    with open("apps_matrix.json", "w") as f:
        f.write(json.dumps(apps_matrix))
    with open("run_for_devices.json", "w") as f:
        f.write(run_for_devices)

    print(f"run_for_devices: {run_for_devices}")
    print("Apps to test:")
    print(json.dumps(apps_matrix, indent=2))
    print("Build matrix:")
    print(json.dumps(build_matrix, indent=2))


if __name__ == "__main__":
    main()
