from ledgered.github import GitHubLedgerHQ, NoManifestException, Condition
from github.GithubException import GithubException

import sys
import json

if len(sys.argv) != 2:
    print("Usage: get_rust_apps.py <github_token>")
    sys.exit(1)

ledger_devices = ["nanos+", "nanox", "stax", "flex"]
filtered_apps = ["app-kadena-legacy", "app-pocket"]

# Retrieve all apps on LedgerHQ GitHub organization
token = sys.argv[1]
gh = GitHubLedgerHQ(token)
apps=gh.apps.filter(archived=Condition.WITHOUT)

rust_apps = []
exclude_apps = []
# loop all apps in gh.apps
for app in apps:
    try: 
        manifest = app.manifest    
    except NoManifestException as e:
        pass
    except GithubException as e:
        pass
    else:
        # Filter out apps that are Rust based
        if manifest.app.sdk == "rust":
                rust_apps.append(app.name)
                # filter out app for specific devices
                for d in ledger_devices:
                    if d not in manifest.app.devices or app.name in filtered_apps:
                        exclude_apps.append({"app-name": app.name, "device": d})

# save the list of Rust apps in a json format:
with open("rust_apps.json", "w") as f:
    f.write(json.dumps(rust_apps))

# save the list of Excluded apps in a json format:
with open("exclude_apps.json", "w") as f:
    f.write(json.dumps(exclude_apps))

# save the list of Ledger devices in a json format:
with open("ledger_devices.json", "w") as f:
    f.write(json.dumps(ledger_devices))