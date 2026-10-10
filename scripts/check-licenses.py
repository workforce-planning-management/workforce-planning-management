#!/usr/bin/env python3
"""Check that the license fields and the LICENSE/ directory agree.

Run from anywhere: `python3 scripts/check-licenses.py`. Exits 1 on a mismatch.

For each subproject manifest (`Cargo.toml`, `package.json`) it reads the SPDX
`OR` expression, checks that every option has its full-text file in `LICENSE/`,
and checks that the root `LICENSE.md` and `LICENSE/index.md` state the same expression.
"""
import json
import os
import re
import subprocess
import sys

ROOT = subprocess.run(
    ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True
).stdout.strip()
TEXTS = {
    "MIT": "LICENSE-MIT",
    "Apache-2.0": "LICENSE-APACHE",
    "BSD-3-Clause": "LICENSE-BSD-3-CLAUSE",
    "GPL-2.0-only": "LICENSE-GPL-2.0",
    "GPL-3.0-only": "LICENSE-GPL-3.0",
}
API = "workforce-planning-management-api-with-rust"
UI = "workforce-planning-management-ui-with-svelte"


def manifests():
    with open(os.path.join(ROOT, API, "Cargo.toml"), encoding="utf-8") as handle:
        cargo = re.search(r'^license\s*=\s*"([^"]+)"', handle.read(), re.M).group(1)
    with open(os.path.join(ROOT, UI, "package.json"), encoding="utf-8") as handle:
        package = json.load(handle)["license"]
    return {f"{API}/Cargo.toml": cargo, f"{UI}/package.json": package}


def main():
    problems = []
    summaries = {}
    for name in ("LICENSE.md", os.path.join("LICENSE", "index.md")):
        path = os.path.join(ROOT, name)
        if not os.path.isfile(path):
            problems.append(f"{name} is missing")
            continue
        with open(path, encoding="utf-8") as handle:
            summaries[name] = handle.read()
    for manifest, expression in manifests().items():
        for name, text in summaries.items():
            if expression not in text:
                problems.append(f"{name} does not state `{expression}` ({manifest})")
        for option in (part.strip() for part in expression.split(" OR ")):
            name = TEXTS.get(option)
            if name is None:
                problems.append(f"{manifest}: no text file known for `{option}`")
            elif not os.path.isfile(os.path.join(ROOT, "LICENSE", name)):
                problems.append(f"{manifest}: LICENSE/{name} is missing (for `{option}`)")
            elif os.path.getsize(os.path.join(ROOT, "LICENSE", name)) < 900:
                problems.append(f"LICENSE/{name} is too short to be a license text")
    for problem in problems:
        print(problem)
    print(f"{len(problems)} license problem(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
