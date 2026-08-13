#!/usr/bin/env python3
"""Write the GitHub Actions job summary for a release run.

Run names are fixed at trigger time — ``Release (minor)`` is set before the
version bump even runs — so the actual released version (``v0.27.0``) is
invisible from the Actions list.  This script writes it, plus the bump type
and the previous version for context, to ``$GITHUB_STEP_SUMMARY`` so the
summary tab shows what was really released.

Usage (inside the release workflow)::

    python generate_release_summary.py --version 0.27.0 --bump minor

Outside Actions (``$GITHUB_STEP_SUMMARY`` unset) the summary is printed to
stdout instead, which makes the script testable locally.
"""

import argparse
import os
import subprocess
import sys


def run(cmd):
    return subprocess.run(cmd, capture_output=True, text=True, check=False)


def previous_version(current, repo):
    """Latest version tag strictly **before** *current*, by semver order.

    Tags are sorted descending (``--sort=-v:refname``), so the entry
    immediately after *current* is its predecessor.  When *current* hasn't
    been tagged yet (``bump = none``), the very first tag is the predecessor.
    Returns ``None`` when there are no prior tags.
    """
    result = run(["git", "-C", repo, "tag", "--sort=-v:refname"])
    if result.returncode != 0:
        return None
    tags = [t.strip().lstrip("v") for t in result.stdout.splitlines() if t.strip()]
    if not tags:
        return None
    if current in tags:
        idx = tags.index(current)
        return tags[idx + 1] if idx + 1 < len(tags) else None
    return tags[0]


def main():
    parser = argparse.ArgumentParser(description="Write the release job summary.")
    parser.add_argument("--version", required=True, help="Released version, e.g. 0.27.0")
    parser.add_argument("--bump", required=True, help="Bump type: patch/minor/major/none")
    args = parser.parse_args()

    repo_path = os.environ.get("GITHUB_WORKSPACE", ".")
    prev = previous_version(args.version, repo_path)

    gh_repo = os.environ.get("GITHUB_REPOSITORY", "emberglazee/Hearts-of-Modding")
    prerelease = "-" in args.version
    release_url = f"https://github.com/{gh_repo}/releases/tag/v{args.version}"
    bump_label = args.bump.capitalize() if args.bump != "none" else "None (manual bump)"

    lines = [
        f"## 🚀 Released v{args.version}\n",
        f"- **Bump type:** {bump_label}",
    ]
    if prev:
        lines.append(f"- **Previous version:** `v{prev}`")
    lines.append(f"- **Prerelease:** {'Yes' if prerelease else 'No'}")
    lines.append(f"- **Release:** [v{args.version}]({release_url})")
    lines.append("")  # trailing newline so the markdown renders cleanly

    output = "\n".join(lines)

    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary_path:
        with open(summary_path, "a") as f:
            f.write(output)
        print("Wrote release summary to $GITHUB_STEP_SUMMARY", file=sys.stderr)
    else:
        print(output)


if __name__ == "__main__":
    main()
