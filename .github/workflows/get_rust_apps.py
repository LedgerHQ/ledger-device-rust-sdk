from ledgered.github import GitHubLedgerHQ, NoManifestException, Condition
from github.GithubException import GithubException

import sys
import json

if len(sys.argv) != 2:
    print("Usage: get_rust_apps.py <github_token>")
    sys.exit(1)

# Excluded Rust apps
# app-kadena-legacy: has been replaced by app-kadena
# app-pocket: does not build (Obsidians' Alamgu issue)
excluded_apps = ["app-kadena-legacy", "app-pocket"]

# Excluded devices
excluded_devices = ["nanos"]

# Retrieve all public apps on LedgerHQ GitHub organization
token = sys.argv[1]
gh = GitHubLedgerHQ(token)
apps=gh.apps.filter(private=Condition.WITHOUT, archived=Condition.WITHOUT)

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
            if app.name not in excluded_apps:
                for d in manifest.app.devices:
                    if d not in excluded_devices:
                        rust_apps.append({"app-name": app.name, "device": d})

# save the list of (apps, device) pairs to build in a json format:
with open("rust_apps.json", "w") as f:
    f.write(json.dumps(rust_apps))