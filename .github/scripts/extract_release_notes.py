#!/usr/bin/env python3
"""Extract release notes from CHANGELOG.md for the current package version.

Minor bump (patch=0): returns just that version's section.
Patch bump (patch>0): returns the patch section + its parent minor section.
"""

import json
import re
import sys
from pathlib import Path


def parse_changelog(path):
    """Return {version: (full_header, body)}.

    `version` is `vX.Y.Z` (lookup key); `full_header` is the whole header line
    after `## ` (e.g. `[v0.25.1] - 2026-08-05 (Hotfix)`), so the full version
    header can be reproduced verbatim in the output.
    """
    text = path.read_text()
    pattern = r'^## (\[v\d+\.\d+\.\d+\][^\n]*)(.*?)(?=^## \[|\Z)'
    matches = re.findall(pattern, text, re.MULTILINE | re.DOTALL)
    return {re.match(r'\[(v\d+\.\d+\.\d+)\]', header).group(1): (header, body.strip()) for header, body in matches}


def load_version(path):
    return json.loads(path.read_text())["version"]


def is_patch(version):
    _, _, patch = map(int, version.split("."))
    return patch > 0


def minor_version_of(version):
    major, minor, _ = map(int, version.split("."))
    return f"{major}.{minor}.0"


def main():
    repo = Path(__file__).resolve().parents[2]
    changelog = repo / "CHANGELOG.md"
    package_json = repo / "client" / "package.json"

    version = load_version(package_json)
    sections = parse_changelog(changelog)

    current_key = f"v{version}"
    if current_key not in sections:
        print(f"error: no changelog entry for {current_key}", file=sys.stderr)
        sys.exit(1)

    current_header, current_body = sections[current_key]
    out = f"## {current_header}\n\n{current_body}"

    if is_patch(version):
        minor_key = f"v{minor_version_of(version)}"
        if minor_key in sections and minor_key != current_key:
            minor_header, minor_body = sections[minor_key]
            out += f"\n\n## {minor_header}\n\n{minor_body}"

    print(out)


if __name__ == "__main__":
    main()
