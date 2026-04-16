window.BENCHMARK_DATA = {
  "lastUpdate": 1776380804344,
  "repoUrl": "https://github.com/OpenDevicePartnership/odp-embedded-controller",
  "entries": {
    "dev-npcx": [
      {
        "commit": {
          "author": {
            "email": "tcdknutson@gmail.com",
            "name": "Dylan Knutson",
            "username": "dymk"
          },
          "committer": {
            "email": "tcdknutson@gmail.com",
            "name": "Dylan Knutson",
            "username": "dymk"
          },
          "distinct": true,
          "id": "f80d37ffcc859aa77e0f70cbe2081eb7cccb9cb1",
          "message": "benchmark: push gh-pages with GITHUB_TOKEN instead of deploy key\n\nThe upstream workflow expects a DEPLOY_KEY secret for SSH auth; that\nsecret doesn't exist in this repo, so actions/checkout silently falls\nback to HTTPS + the ephemeral GITHUB_TOKEN extraheader. The explicit\n`-c http.https://github.com/.extraheader=` on the manual push then\nclears that token, leaving the push with no credentials:\n\n  fatal: could not read Username for 'https://github.com':\n    No such device or address\n\nThe job already declares `permissions: contents: write`, so\nGITHUB_TOKEN has enough scope to push benchmark data to gh-pages.\nDrop the ssh-key argument from checkout and let the default auth\nflow the benchmark action sets up survive into the push step.\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-04-16T15:50:22-07:00",
          "tree_id": "00b4247c962001a0bd26e47036905a4599ec39af",
          "url": "https://github.com/OpenDevicePartnership/odp-embedded-controller/commit/f80d37ffcc859aa77e0f70cbe2081eb7cccb9cb1"
        },
        "date": 1776380749142,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "Binary Size",
            "value": 78.43,
            "unit": "KiB",
            "extra": "RAM Size: 7.48 KiB\nDependency Count: 356\nVersion: rustc 1.95.0 (59807616e 2026-04-14)"
          }
        ]
      }
    ],
    "dev-imxrt": [
      {
        "commit": {
          "author": {
            "email": "tcdknutson@gmail.com",
            "name": "Dylan Knutson",
            "username": "dymk"
          },
          "committer": {
            "email": "tcdknutson@gmail.com",
            "name": "Dylan Knutson",
            "username": "dymk"
          },
          "distinct": true,
          "id": "f80d37ffcc859aa77e0f70cbe2081eb7cccb9cb1",
          "message": "benchmark: push gh-pages with GITHUB_TOKEN instead of deploy key\n\nThe upstream workflow expects a DEPLOY_KEY secret for SSH auth; that\nsecret doesn't exist in this repo, so actions/checkout silently falls\nback to HTTPS + the ephemeral GITHUB_TOKEN extraheader. The explicit\n`-c http.https://github.com/.extraheader=` on the manual push then\nclears that token, leaving the push with no credentials:\n\n  fatal: could not read Username for 'https://github.com':\n    No such device or address\n\nThe job already declares `permissions: contents: write`, so\nGITHUB_TOKEN has enough scope to push benchmark data to gh-pages.\nDrop the ssh-key argument from checkout and let the default auth\nflow the benchmark action sets up survive into the push step.\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-04-16T15:50:22-07:00",
          "tree_id": "00b4247c962001a0bd26e47036905a4599ec39af",
          "url": "https://github.com/OpenDevicePartnership/odp-embedded-controller/commit/f80d37ffcc859aa77e0f70cbe2081eb7cccb9cb1"
        },
        "date": 1776380803229,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "Binary Size",
            "value": 81.91,
            "unit": "KiB",
            "extra": "RAM Size: 10.62 KiB\nDependency Count: 378\nVersion: rustc 1.95.0 (59807616e 2026-04-14)"
          }
        ]
      }
    ]
  }
}