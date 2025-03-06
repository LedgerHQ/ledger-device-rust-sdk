from ledgered.github import GitHubLedgerHQ, NoManifestException
from github.GithubException import GithubException
import sys

if len(sys.argv) != 2:
    print("Usage: get_rust_apps.py <github_token>")
    sys.exit(1)

token = sys.argv[1]
gh = GitHubLedgerHQ(token)
apps=gh.apps

rust_apps = []
# loop all apps in gh.apps
for app in apps:
    try: 
        manifest = app.manifest    
    except NoManifestException as e:
        pass
    except GithubException as e:
        pass
    else:
        if manifest.app.sdk == "rust":
            # store app_name in a list
            rust_apps.append(app.name)

# print the list of rust apps in between []
print(rust_apps)
