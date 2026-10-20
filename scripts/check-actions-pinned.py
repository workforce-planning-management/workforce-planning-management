#!/usr/bin/env python3
"""Fail if a GitHub Actions workflow uses an action that is not pinned to a full commit SHA.

A tag such as `@v4` can be moved to different code after it is reviewed; a 40-character commit
SHA cannot (WPM-R119, WPM-T194). Write `uses: owner/repo@<sha> # v4` so a person can still read
the version; Dependabot updates both. Local actions (`./...`) and `docker://` references with a
digest are fine.

    python3 scripts/check-actions-pinned.py
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
USES = re.compile(r"^\s*(?:-\s+)?uses:\s*(\S+)")
PINNED = re.compile(r"^[\w.-]+/[\w./-]+@[0-9a-f]{40}$")


def main() -> int:
    bad = []
    for path in sorted((ROOT / ".github").rglob("*.y*ml")):
        for number, line in enumerate(path.read_text().splitlines(), 1):
            m = USES.match(line)
            if not m:
                continue
            ref = m.group(1)
            if ref.startswith("./") or re.search(r"^docker://.+@sha256:[0-9a-f]{64}$", ref):
                continue
            if not PINNED.match(ref):
                bad.append(f"{path.relative_to(ROOT)}:{number}: {ref} is not pinned to a commit SHA")
    for line in bad:
        print(line)
    print("all actions are pinned" if not bad else f"{len(bad)} unpinned action(s)")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
