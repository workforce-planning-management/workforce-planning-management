#!/usr/bin/env python3
"""Check that the license fields and the LICENSES/ directory agree.

Run from anywhere: `python3 scripts/check-licenses.py`. Exits 1 on a mismatch.

For each subproject manifest (`Cargo.toml`, `package.json`) it reads the SPDX
`OR` expression, checks that every option has `LICENSES/<lowercase-spdx-id>/index.md`
holding a full text, and checks that the root `LICENSE.md` states the same
expression. It also checks that `LICENSES/` holds only those directories (each
with only `index.md`), that the old `LICENSE/` directory is gone, and that no
other license text file (`LICENSE-*`, `COPYING`, ...) sits outside `LICENSES/`.
"""
import json
import os
import re
import subprocess
import sys

ROOT = subprocess.run(
    ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True
).stdout.strip()
KNOWN = {
    "MIT",
    "Apache-2.0",
    "BSD-3-Clause",
    "GPL-2.0-only",
    "GPL-3.0-only",
    "OGL-UK-3.0",
    "CC-BY-NC-SA-4.0",
}
API = "workforce-planning-management-service-with-rust"
UI = "workforce-planning-management-ui-with-svelte"
MATCHER = "workforce-planning-management-matcher-with-rust"
SITE = "workforce-planning-management.github.io"


def manifests():
    with open(os.path.join(ROOT, API, "Cargo.toml"), encoding="utf-8") as handle:
        cargo = re.search(r'^license\s*=\s*"([^"]+)"', handle.read(), re.M).group(1)
    with open(os.path.join(ROOT, UI, "package.json"), encoding="utf-8") as handle:
        package = json.load(handle)["license"]
    with open(os.path.join(ROOT, MATCHER, "Cargo.toml"), encoding="utf-8") as handle:
        matcher = re.search(r'^license\s*=\s*"([^"]+)"', handle.read(), re.M).group(1)
    with open(os.path.join(ROOT, SITE, "package.json"), encoding="utf-8") as handle:
        site = json.load(handle)["license"]
    return {
        f"{API}/Cargo.toml": cargo,
        f"{UI}/package.json": package,
        f"{MATCHER}/Cargo.toml": matcher,
        f"{SITE}/package.json": site,
    }


def stray_texts():
    """License-named files outside LICENSES/ that git tracks or would commit."""
    listed = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        capture_output=True, text=True, check=True, cwd=ROOT,
    ).stdout.split("\0")
    found = []
    for rel in filter(None, listed):
        name = os.path.basename(rel).lower()
        stem = name.split(".")[0]
        named = any(
            stem == base or stem.startswith(base + "-")
            for base in ("license", "licence", "copying")
        )
        in_licenses = rel.startswith("LICENSES/")
        if named and not in_licenses and rel != "LICENSE.md" and os.path.exists(os.path.join(ROOT, rel)):
            found.append(rel)
    return found


def main():
    problems = []
    path = os.path.join(ROOT, "LICENSE.md")
    summary = ""
    if os.path.isfile(path):
        with open(path, encoding="utf-8") as handle:
            summary = handle.read()
    else:
        problems.append("LICENSE.md is missing")
    if os.path.exists(os.path.join(ROOT, "LICENSE")):
        problems.append("LICENSE/ must not exist: texts live in LICENSES/<id>/index.md")
    for manifest, expression in manifests().items():
        if summary and expression not in summary:
            problems.append(f"LICENSE.md does not state `{expression}` ({manifest})")
        for option in (part.strip() for part in expression.split(" OR ")):
            if option not in KNOWN:
                problems.append(f"{manifest}: unknown license `{option}`")
                continue
            rel = os.path.join("LICENSES", option.lower(), "index.md")
            full = os.path.join(ROOT, rel)
            if not os.path.isfile(full):
                problems.append(f"{manifest}: {rel} is missing (for `{option}`)")
                continue
            with open(full, encoding="utf-8") as handle:
                text = handle.read()
            if "```text" not in text or len(text) < 900:
                problems.append(f"{rel} holds no full license text in a text fence")
            if f"LICENSES/{option.lower()}/index.md" not in summary:
                problems.append(f"LICENSE.md does not link {rel}")
    licenses_dir = os.path.join(ROOT, "LICENSES")
    if os.path.isdir(licenses_dir):
        allowed = {option.lower() for option in KNOWN}
        for entry in sorted(os.listdir(licenses_dir)):
            full = os.path.join(licenses_dir, entry)
            if entry not in allowed or not os.path.isdir(full):
                problems.append(f"LICENSES/{entry} is not a known license directory")
            else:
                # `index.md`, and optionally a `README.md` that is only a symlink to it (so GitHub shows the text), never a second copy.
                held = sorted(os.listdir(full))
                readme = os.path.join(full, "README.md")
                readme_ok = (
                    "README.md" in held
                    and os.path.islink(readme)
                    and os.readlink(readme) == "index.md"
                )
                expected = ["README.md", "index.md"] if readme_ok else ["index.md"]
                if held != expected:
                    problems.append(
                        f"LICENSES/{entry} must hold only index.md (and optionally a README.md symlink to it)"
                    )
    else:
        problems.append("LICENSES/ is missing")
    for rel in stray_texts():
        problems.append(f"{rel}: license text outside LICENSES/")
    for problem in problems:
        print(problem)
    print(f"{len(problems)} license problem(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
