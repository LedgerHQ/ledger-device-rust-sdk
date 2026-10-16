from ledgered.github import GitHubLedgerHQ, NoManifestException, Condition
from github.GithubException import GithubException

import sys
import json

if len(sys.argv) not in (2, 3):
    print("Usage: get_rust_apps.py <github_token> [<private_repo_suffix>]")
    sys.exit(1)

# Excluded Rust apps
# app-kadena-legacy: has been replaced by app-kadena
# app-pocket: does not build (Obsidians' Alamgu issue)
excluded_apps = ["app-kadena-legacy", "app-pocket"]

# Retrieve public apps, or private ones ending with the suffix
token = sys.argv[1]
suffix = sys.argv[2] if len(sys.argv) == 3 else ""
gh = GitHubLedgerHQ(token)
if suffix:
    apps = [app for app in gh.apps.filter(private=Condition.ONLY, archived=Condition.WITHOUT)
            if app.name.endswith(suffix)]
else:
    apps = gh.apps.filter(private=Condition.WITHOUT, archived=Condition.WITHOUT)

rust_apps_name_and_device = []
rust_apps_name = []
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
            if app.name.removesuffix(suffix) not in excluded_apps:
                rust_apps_name.append({"app-name": app.name})
                for d in manifest.app.devices:
                    rust_apps_name_and_device.append({"app-name": app.name, "device": d})

# save the list of (apps, device) pairs to build in a json format:
with open("rust_apps.json", "w") as f:
    f.write(json.dumps(rust_apps_name_and_device))
# save the list of rust app names in a json format:
with open("rust_apps_names.json", "w") as f:
    f.write(json.dumps(rust_apps_name))