#!/usr/bin/env python3
"""Generate the GitHub release description for the current package version.

The description is the changelog section(s) for the version plus a
contributor medal list covering everything since the previous release tag:

    # 🎖️ Contributors since last release

    - 🥇 @emberglazee: 29 commits
    - 🥈 @Copilot: 4 commits
    - 🥉 @jiangji0721: 2 commits
    - 🏅 @github-actions: 1 commit

Changelog rules:
- Minor bump (patch == 0): returns just that version's section.
- Patch bump (patch > 0): returns the patch section + its parent minor section.

Contributor rules (one credit per commit in <previous tag>..HEAD, each commit
counted exactly once):
- Non-merge commits count for their author.
- A merge commit whose `merge_commit_sha` matches a merged pull request
  counts for the PR AUTHOR, not the merger — merged PRs are found via the
  pulls API (`state=closed` + `merged_at`), matched by exact merge commit
  SHA. The `Merge pull request #N` message is NOT parsed: it can reference
  PRs from another repository context and is wrong as data (seen in the
  wild here — a commit saying "Merge pull request #1 from ..." whose real
  PR #1 is an unrelated dependabot bump).
- A merge commit with no matching PR record (e.g. `Merge branch`, or a merge
  imported from another repo) counts for the merger.
- Handles come from the GitHub API (`author.login`), which is authoritative
  and merges the multiple local git identities of one account. Without a
  token the script falls back to parsing git author emails — noreply
  addresses carry the login (`12345+login@users.noreply.github.com`);
  anything else falls back to the author's git name. PR authors cannot be
  resolved offline, so merge commits are skipped with a warning.
- The top three contributors get 🥇/🥈/🥉; everyone else gets 🏅 — any
  number of contributions earns an honorable mention.

Usage:
    generate_release_description.py [--prev-tag TAG] [--local]

Options:
    --prev-tag TAG   count contributions since an explicit tag (default: the
                     newest tag reachable from HEAD, via `git describe`)
    --local          skip the GitHub API; derive handles from git data only

API notes (all verified against this repo's data):
- `gh api` with `-f` fields defaults to POST — query params go in the URL.
- The compare endpoint 404s on the literal ref `HEAD`; pass an explicit SHA.
- `state=merged` is not a valid pulls-list filter (silently returns []); use
  `state=closed` and keep PRs that have `merged_at`.
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

MEDALS = ("🥇", "🥈", "🥉")
HONORABLE_MENTION = "🏅"
CONTRIBUTOR_HEADER = "# 🎖️ Contributors since last release"

# `12345+login@users.noreply.github.com` or `login@users.noreply.github.com`
NOREPLY_RE = re.compile(r"^(?:\d+\+)?([^@+]+)@users\.noreply\.github\.com$")


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


def run(cmd):
    return subprocess.run(cmd, capture_output=True, text=True, check=False)


def previous_release_tag(repo):
    """Newest tag reachable from HEAD (`git describe`), or None."""
    result = run(["git", "-C", repo, "describe", "--tags", "--abbrev=0"])
    if result.returncode != 0:
        return None
    return result.stdout.strip()


def github_repository(repo):
    """`owner/name` from the environment, else parsed from the origin remote."""
    repo_env = os.environ.get("GITHUB_REPOSITORY")
    if repo_env:
        return repo_env
    result = run(["git", "-C", repo, "remote", "get-url", "origin"])
    if result.returncode != 0:
        return None
    url = result.stdout.strip()
    match = re.search(r"(?:github\.com[:\/]|git@github\.com:)([\w.-]+/[\w.-]+?)(?:\.git)?$", url)
    return match.group(1) if match else None


def handle_from_email(email):
    """Derive a display handle from a commit email, or None."""
    match = NOREPLY_RE.match(email)
    return match.group(1) if match else None


def display_handle(login):
    """Strip the `[bot]` suffix for readability (@dependabot[bot] -> @dependabot)."""
    return login[:-5] if login.endswith("[bot]") else login


def _login_of(author, commit_author):
    """GitHub login for a commit, or a derived fallback when unlinked."""
    login = (author or {}).get("login")
    if login:
        return login
    email = commit_author["email"]
    return handle_from_email(email) or commit_author["name"]


def _sort_entries(counts):
    """[(handle, count)] sorted by count desc, handle asc (stable output)."""
    return sorted(counts.items(), key=lambda kv: (-kv[1], kv[0].lower()))


def _api_json(args, what=""):
    result = run(["gh", "api", *args])
    if result.returncode != 0:
        print(f"warning: gh api failed ({what}): {result.stderr.strip()}", file=sys.stderr)
        return None
    return json.loads(result.stdout)


def compare_commits(owner_repo, prev_tag, head):
    """All commits in `prev_tag...head`, paginated. Returns [] only on success."""
    commits, page = [], 1
    total = None
    while True:
        data = _api_json([f"repos/{owner_repo}/compare/{prev_tag}...{head}?page={page}&per_page=100"], "compare")
        if data is None:
            return None
        total = data["total_commits"]
        commits.extend(data["commits"])
        if not data["commits"] or len(commits) >= total:
            break
        page += 1
    if len(commits) < total:
        print(f"warning: compare truncated ({len(commits)} of {total} commits)", file=sys.stderr)
    return commits


def merged_pr_authors_by_sha(owner_repo):
    """{merge_commit_sha: pr_author_login} for every merged PR (paginated)."""
    by_sha, page = {}, 1
    while True:
        prs = _api_json([f"repos/{owner_repo}/pulls?state=closed&per_page=100&page={page}"], "pulls")
        if prs is None:
            return None
        for pr in prs:
            if pr.get("merged_at") and pr.get("merge_commit_sha"):
                by_sha[pr["merge_commit_sha"]] = pr["user"]["login"]
        if not prs:
            break
        page += 1
    return by_sha


def contributors_from_api(owner_repo, prev_tag, repo):
    """[(display_handle, count)] — API-authoritative version.

    Non-merge commits count for their author; merged PRs count for the PR
    author (joined by `merge_commit_sha`); orphan merges count for the merger.
    """
    head_result = run(["git", "-C", repo, "rev-parse", "HEAD"])
    if head_result.returncode != 0:
        return None
    head = head_result.stdout.strip()

    commits = compare_commits(owner_repo, prev_tag, head)
    pr_by_sha = merged_pr_authors_by_sha(owner_repo)
    if commits is None or pr_by_sha is None:
        return None

    counts = {}
    for commit in commits:
        commit_author = commit["commit"]["author"]
        if len(commit.get("parents", [])) > 1:
            login = pr_by_sha.get(commit["sha"])  # PR author when the merge is a real PR merge
            if not login:
                login = _login_of(commit.get("author"), commit_author)  # orphan merge: the merger
        else:
            login = _login_of(commit.get("author"), commit_author)
        counts[display_handle(login)] = counts.get(display_handle(login), 0) + 1
    return _sort_entries(counts)


def contributors_from_git(repo, prev_tag):
    """[(display_handle, count)] from `git log`, deduped by email.

    Offline fallback: merge commits are skipped entirely — PR authors can't
    be resolved without the API.
    """
    result = run([
        "git", "-C", repo, "log", "--no-merges",
        "--format=%H%x00%an%x00%ae", f"{prev_tag}..HEAD",
    ])
    if result.returncode != 0:
        return None

    merges = run(["git", "-C", repo, "rev-list", "--merges", "--count", f"{prev_tag}..HEAD"])
    if merges.returncode == 0 and merges.stdout.strip() not in ("", "0"):
        print("warning: merged PR authors omitted (no GitHub API in --local mode)", file=sys.stderr)

    # email -> {name -> count}  (one account may use several git names)
    identities = {}
    for line in result.stdout.splitlines():
        _, name, email = line.split("\x00", 2)
        by_name = identities.setdefault(email, {})
        by_name[name] = by_name.get(name, 0) + 1

    counts = {}
    for email, by_name in identities.items():
        login = handle_from_email(email)
        if not login:
            login, _ = max(by_name.items(), key=lambda kv: kv[1])  # most-used name
        counts[display_handle(login)] = counts.get(display_handle(login), 0) + sum(by_name.values())
    return _sort_entries(counts)


def render_contributors(entries):
    lines = [CONTRIBUTOR_HEADER, ""]
    for rank, (handle, count) in enumerate(entries):
        medal = MEDALS[rank] if rank < len(MEDALS) else HONORABLE_MENTION
        noun = "commit" if count == 1 else "commits"
        lines.append(f"- {medal} @{handle}: {count} {noun}")
    return "\n".join(lines)


def changelog_part(version, sections):
    current_key = f"v{version}"
    current_header, current_body = sections[current_key]
    out = f"## {current_header}\n\n{current_body}"

    if is_patch(version):
        minor_key = f"v{minor_version_of(version)}"
        if minor_key in sections and minor_key != current_key:
            minor_header, minor_body = sections[minor_key]
            out += f"\n\n## {minor_header}\n\n{minor_body}"

    return out


def main():
    args = sys.argv[1:]
    prev_tag_override = None
    local_only = False
    if "--prev-tag" in args:
        prev_tag_override = args[args.index("--prev-tag") + 1]
    if "--local" in args:
        local_only = True

    repo = Path(__file__).resolve().parents[2]
    changelog = repo / "CHANGELOG.md"
    package_json = repo / "client" / "package.json"

    version = load_version(package_json)
    sections = parse_changelog(changelog)

    if f"v{version}" not in sections:
        print(f"error: no changelog entry for v{version}", file=sys.stderr)
        sys.exit(1)

    parts = []

    prev_tag = prev_tag_override or previous_release_tag(repo)
    if prev_tag:
        if local_only:
            entries = contributors_from_git(repo, prev_tag)
        else:
            owner_repo = github_repository(repo)
            entries = contributors_from_api(owner_repo, prev_tag, repo) if owner_repo else None
            if entries is None:
                entries = contributors_from_git(repo, prev_tag)  # offline fallback
        if entries:
            parts.append(render_contributors(entries))

    parts.append(changelog_part(version, sections))
    print("\n\n".join(parts))


if __name__ == "__main__":
    main()