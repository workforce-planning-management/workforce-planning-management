#!/usr/bin/env python3
"""Fail if a GitHub Actions job has no `timeout-minutes` (WPM-T508).

GitHub's own limit is six hours, so a hung step holds a runner, and the merge it gates, for that long.
Each job in `.github/workflows/*.yml` sets its own limit, about twice its slowest recent run and at
least ten minutes. A job that calls a reusable workflow (`uses:` at the job's level) runs no steps
of its own, so it is skipped.

    python3 scripts/check-job-timeouts.py
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
JOB = re.compile(r"^  ([A-Za-z0-9_-]+):\s*$")
KEY = re.compile(r"^    ([A-Za-z0-9_-]+):")


def missing(path: Path) -> list[str]:
    """The jobs in one workflow file with neither `timeout-minutes` nor a job-level `uses`."""
    bad, in_jobs, job, keys = [], False, None, set()

    def close() -> None:
        if job and "timeout-minutes" not in keys and "uses" not in keys:
            bad.append(job)

    for line in path.read_text().splitlines():
        if re.match(r"^\S", line):
            if in_jobs:
                close()
                job = None
            in_jobs = line.rstrip() == "jobs:"
            continue
        if not in_jobs:
            continue
        if m := JOB.match(line):
            close()
            job, keys = m.group(1), set()
        elif job and (m := KEY.match(line)):
            keys.add(m.group(1))
    if in_jobs:
        close()
    return bad


def main() -> int:
    problems = []
    for path in sorted((ROOT / ".github" / "workflows").glob("*.y*ml")):
        problems += [f"{path.relative_to(ROOT)}: job `{job}` has no timeout-minutes" for job in missing(path)]
    for problem in problems:
        print(problem)
    if problems:
        return 1
    print("every job has a time limit")
    return 0


if __name__ == "__main__":
    sys.exit(main())
