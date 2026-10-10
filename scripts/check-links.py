#!/usr/bin/env python3
"""Check that every relative Markdown link in the repository resolves.

Run from anywhere: `python3 scripts/check-links.py`. Exits 1 and lists each
dead link. External (http, https, mailto) links are not fetched, so this runs
offline and never flakes; a link to a sibling repository must be plain text,
not a link, because it cannot resolve in a fresh clone.

Skipped on purpose: `.sota/` (scan records) and two spec folders that are
reference material written for other repositories, whose example links point
at files that exist only there.
"""
import os
import re
import subprocess
import sys

ROOT = subprocess.run(
    ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True
).stdout.strip()
SKIP = (".sota", "spec/special-files-for-public-repos", "spec/oxford-spelling")
LINK = re.compile(r"\[([^\]]*)\]\(([^)\s#]+)(#[^)]*)?\)")


def markdown_files():
    out = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "*.md"],
        capture_output=True, text=True, check=True, cwd=ROOT,
    ).stdout.split()
    return sorted(f for f in set(out) if not f.startswith(SKIP))


def main():
    dead = []
    for name in markdown_files():
        path = os.path.join(ROOT, name)
        if not os.path.exists(path):
            continue
        with open(path, encoding="utf-8") as handle:
            for number, line in enumerate(handle, 1):
                for match in LINK.finditer(line):
                    target = match.group(2)
                    if re.match(r"(https?:|mailto:)", target):
                        continue
                    resolved = os.path.normpath(os.path.join(os.path.dirname(path), target))
                    if not os.path.exists(resolved):
                        dead.append(f"{name}:{number}: {target}")
    for item in dead:
        print(item)
    print(f"{len(dead)} dead link(s)")
    return 1 if dead else 0


if __name__ == "__main__":
    sys.exit(main())
