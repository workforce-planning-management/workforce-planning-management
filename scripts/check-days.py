#!/usr/bin/env python3
"""Check that every duration counted in days says which kind of day.

The rule is `spec/calendar-days-or-business-days`: a number of days is always
"calendar days" or "business days", never a bare "days". Run from anywhere:
`python3 scripts/check-days.py`. Exits 1 and lists each offending line.

Checked: Markdown, and the English UI strings. Not checked: code identifiers
(`days_ahead`, `horizon_days`), other units, and a single day that is not a
duration ("the last day").
"""
import os
import re
import subprocess
import sys

ROOT = subprocess.run(
    ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True
).stdout.strip()
SKIP = (".sota", "spec/calendar-days-or-business-days")
EXTRA = ["workforce-planning-management-ui-with-svelte/content/locales/en-001/ui.json"]
# A number (or N, a range, ±n) directly followed by "days", an "n-day"
# adjective, or "whole days".
BARE = re.compile(r"(?<![\w-])(?:±)?(?:\d+[–-])?(?:\d+|N) days?\b|\b\d+-day\b|\bwhole days\b")


def files():
    out = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "*.md"],
        capture_output=True, text=True, check=True, cwd=ROOT,
    ).stdout.split()
    return sorted({f for f in out if not f.startswith(SKIP)} | set(EXTRA))


def main():
    bad = []
    for name in files():
        path = os.path.join(ROOT, name)
        if not os.path.exists(path):
            continue
        with open(path, encoding="utf-8") as handle:
            for number, line in enumerate(handle, 1):
                # "30 calendar days" never matches: the kind sits between the
                # number and "days", so any match is a bare duration.
                if BARE.search(line):
                    bad.append(f"{name}:{number}: {line.strip()[:110]}")
    for item in bad:
        print(item)
    print(f"{len(bad)} line(s) with a bare number of days")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
