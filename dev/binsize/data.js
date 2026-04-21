window.BENCHMARK_DATA = {
  "lastUpdate": 1776792024692,
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
      },
      {
        "commit": {
          "author": {
            "email": "dymk@users.noreply.github.com",
            "name": "Dylan Knutson",
            "username": "dymk"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4eb2158253ef1bc4dcfa46ed749aca164ab6de07",
          "message": "chore: initialize cargo-vet (#3)\n\n* chore: initialize cargo-vet\n\nInitialize cargo-vet for each of the 3 platform crates (dev-imxrt,\ndev-npcx, dev-qemu). Each platform has its own supply-chain/ store\nwith audits.toml, config.toml, and imports.lock — matching cargo-vet's\nper-workspace design (each platform has its own Cargo.lock and dep\ngraph, so a shared store would not satisfy cargo-vet's policy\nvalidation).\n\nProcess documentation lives at docs/supply-chain.md (single shared\ncopy at repo root rather than duplicated into each store).\n\nImports the OpenDevicePartnership, bytecode-alliance, google, and\nmozilla audit sources, mirroring the embedded-services pattern.\n\nAdds two CI workflows:\n- cargo-vet.yml: matrix per platform, runs 'cargo vet --locked' from\n  each platform/<dev-*>/ directory.\n- cargo-vet-pr-comment.yml: workflow_run-triggered, downloads the\n  per-platform pr-* artifacts via merge-multiple, posts the audit\n  questionnaire on failure and updates to a success message on pass.\n\nAll 3 platforms pass 'cargo vet --locked'.\n\n* chore(cargo-vet): apply review feedback\n\n- pin rust toolchain via dtolnay/rust-toolchain\n- pass --locked to cargo install cargo-vet\n- skip pr-comment job on cancelled upstream runs\n- link docs/supply-chain.md at workflow_run head_sha (works on open PRs)\n- 'Github' -> 'GitHub'",
          "timestamp": "2026-04-21T09:36:46-05:00",
          "tree_id": "d9e241e6676d3f1f9e7f999c3ad3b6a88d4095fb",
          "url": "https://github.com/OpenDevicePartnership/odp-embedded-controller/commit/4eb2158253ef1bc4dcfa46ed749aca164ab6de07"
        },
        "date": 1776782355314,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "Binary Size",
            "value": 78.32,
            "unit": "KiB",
            "extra": "RAM Size: 7.48 KiB\nDependency Count: 356\nVersion: rustc 1.95.0 (59807616e 2026-04-14)"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "kdinelle@microsoft.com",
            "name": "Kurtis Dinelle",
            "username": "kurtjd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "43c220b30d49f9bab9f2da8b72a5ccfa0a5c3aad",
          "message": "platform-common: Downgrade embedded-mcu-hal (#4)",
          "timestamp": "2026-04-21T11:05:13-05:00",
          "tree_id": "51cd7477d05c3b0e4b2afb8ee7b311d7c745744b",
          "url": "https://github.com/OpenDevicePartnership/odp-embedded-controller/commit/43c220b30d49f9bab9f2da8b72a5ccfa0a5c3aad"
        },
        "date": 1776787649929,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "Binary Size",
            "value": 78.32,
            "unit": "KiB",
            "extra": "RAM Size: 7.48 KiB\nDependency Count: 356\nVersion: rustc 1.95.0 (59807616e 2026-04-14)"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "kdinelle@microsoft.com",
            "name": "Kurtis Dinelle",
            "username": "kurtjd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1733a4007743ca2b6cc9144c1f0f81ebdb2818ec",
          "message": "Add platform-common to clippy CI job (#5)",
          "timestamp": "2026-04-21T12:18:54-05:00",
          "tree_id": "4b3cc9dc3d9d00003e1bc30ab4332d882c3d8d1c",
          "url": "https://github.com/OpenDevicePartnership/odp-embedded-controller/commit/1733a4007743ca2b6cc9144c1f0f81ebdb2818ec"
        },
        "date": 1776792024278,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "Binary Size",
            "value": 78.32,
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
      },
      {
        "commit": {
          "author": {
            "email": "dymk@users.noreply.github.com",
            "name": "Dylan Knutson",
            "username": "dymk"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4eb2158253ef1bc4dcfa46ed749aca164ab6de07",
          "message": "chore: initialize cargo-vet (#3)\n\n* chore: initialize cargo-vet\n\nInitialize cargo-vet for each of the 3 platform crates (dev-imxrt,\ndev-npcx, dev-qemu). Each platform has its own supply-chain/ store\nwith audits.toml, config.toml, and imports.lock — matching cargo-vet's\nper-workspace design (each platform has its own Cargo.lock and dep\ngraph, so a shared store would not satisfy cargo-vet's policy\nvalidation).\n\nProcess documentation lives at docs/supply-chain.md (single shared\ncopy at repo root rather than duplicated into each store).\n\nImports the OpenDevicePartnership, bytecode-alliance, google, and\nmozilla audit sources, mirroring the embedded-services pattern.\n\nAdds two CI workflows:\n- cargo-vet.yml: matrix per platform, runs 'cargo vet --locked' from\n  each platform/<dev-*>/ directory.\n- cargo-vet-pr-comment.yml: workflow_run-triggered, downloads the\n  per-platform pr-* artifacts via merge-multiple, posts the audit\n  questionnaire on failure and updates to a success message on pass.\n\nAll 3 platforms pass 'cargo vet --locked'.\n\n* chore(cargo-vet): apply review feedback\n\n- pin rust toolchain via dtolnay/rust-toolchain\n- pass --locked to cargo install cargo-vet\n- skip pr-comment job on cancelled upstream runs\n- link docs/supply-chain.md at workflow_run head_sha (works on open PRs)\n- 'Github' -> 'GitHub'",
          "timestamp": "2026-04-21T09:36:46-05:00",
          "tree_id": "d9e241e6676d3f1f9e7f999c3ad3b6a88d4095fb",
          "url": "https://github.com/OpenDevicePartnership/odp-embedded-controller/commit/4eb2158253ef1bc4dcfa46ed749aca164ab6de07"
        },
        "date": 1776782392769,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "Binary Size",
            "value": 81.79,
            "unit": "KiB",
            "extra": "RAM Size: 10.62 KiB\nDependency Count: 378\nVersion: rustc 1.95.0 (59807616e 2026-04-14)"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "kdinelle@microsoft.com",
            "name": "Kurtis Dinelle",
            "username": "kurtjd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "43c220b30d49f9bab9f2da8b72a5ccfa0a5c3aad",
          "message": "platform-common: Downgrade embedded-mcu-hal (#4)",
          "timestamp": "2026-04-21T11:05:13-05:00",
          "tree_id": "51cd7477d05c3b0e4b2afb8ee7b311d7c745744b",
          "url": "https://github.com/OpenDevicePartnership/odp-embedded-controller/commit/43c220b30d49f9bab9f2da8b72a5ccfa0a5c3aad"
        },
        "date": 1776787702996,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "Binary Size",
            "value": 81.79,
            "unit": "KiB",
            "extra": "RAM Size: 10.62 KiB\nDependency Count: 378\nVersion: rustc 1.95.0 (59807616e 2026-04-14)"
          }
        ]
      }
    ]
  }
}