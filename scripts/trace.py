#!/usr/bin/env python3
"""List the requirements marked built that no test cites (WPM-T512, WPM-R251). Report only: it always exits 0.

A requirement is one defined in `spec/` (a `## WPM-R<n>` heading, or a `- **WPM-R<n>` item). No requirement carries a
"built" mark of its own, so a requirement counts as **built** when a task marked done (`- [x]`) in `spec/tasks.md` cites
it, anywhere in that entry. A requirement counts as **cited by a test** when its id appears in:

- a test file: the service's `tests/`, the matcher's `tests/` and `benches/`, the UI's and the site's `tests/`;
- the test code inside the service's and the matcher's `src/`: from the first `#[cfg(test)]` in a file to its end.

A citation is a name, not a check: a test that names WPM-R123 and asserts nothing still counts. So this measures where
the trace is missing, not how well anything is tested.

    python3 scripts/trace.py            # the counts, then each built requirement no test cites
    python3 scripts/trace.py --json     # the same, as JSON
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
API = ROOT / "workforce-planning-management-service-with-rust"
MATCHER = ROOT / "workforce-planning-management-matcher-with-rust"
UI = ROOT / "workforce-planning-management-ui-with-svelte"
SITE = ROOT / "workforce-planning-management.github.io"

DEFINED = re.compile(r"^(?:##+ |- \*\*)WPM-R(\d+)\b", re.M)
REQ = re.compile(r"\bWPM-R(\d+)\b")
# A range such as "WPM-R101–R104" or "WPM-R1-R3" cites every id in it.
RANGE = re.compile(r"\bWPM-R(\d+)\s*[–-]\s*(?:WPM-)?R(\d+)\b")
ENTRY = re.compile(r"^- \[( |x)\] ")
TEST_SUFFIXES = {".rs", ".ts", ".js", ".mjs", ".svelte", ".py"}
SKIP_DIRS = {"node_modules", "target", ".svelte-kit", "build", "fixtures"}


def ids_in(text: str) -> set[int]:
    found = {int(n) for n in REQ.findall(text)}
    for start, end in RANGE.findall(text):
        a, b = int(start), int(end)
        if a < b <= a + 50:
            found.update(range(a, b + 1))
    return found


def defined() -> set[int]:
    found: set[int] = set()
    for path in (ROOT / "spec").rglob("*.md"):
        found.update(int(n) for n in DEFINED.findall(path.read_text()))
    return found


def built() -> set[int]:
    """The ids cited by a done task entry: the `- [x]` line and its indented continuation."""
    found: set[int] = set()
    done = False
    for line in (ROOT / "spec" / "tasks.md").read_text().splitlines():
        if m := ENTRY.match(line):
            done = m.group(1) == "x"
        elif not line.startswith((" ", "\t")):
            done = False
        if done:
            found.update(ids_in(line))
    return found


def files(base: Path):
    if not base.is_dir():
        return
    for path in base.rglob("*"):
        if path.is_file() and path.suffix in TEST_SUFFIXES and not SKIP_DIRS & set(path.relative_to(base).parts):
            yield path


def cited() -> set[int]:
    found: set[int] = set()
    for base in (API / "tests", MATCHER / "tests", MATCHER / "benches", UI / "tests", SITE / "tests"):
        for path in files(base):
            found.update(ids_in(path.read_text(errors="replace")))
    for base in (API / "src", API / "crates", MATCHER / "src"):
        for path in files(base):
            if path.suffix != ".rs":
                continue
            text = path.read_text(errors="replace")
            at = text.find("#[cfg(test)]")
            if at >= 0:
                found.update(ids_in(text[at:]))
    return found


def main() -> int:
    every, done, tested = defined(), built(), cited()
    built_known = done & every
    untested = sorted(built_known - tested)
    report = {
        "requirements": len(every),
        "built": len(built_known),
        "built_and_cited_by_a_test": len(built_known & tested),
        "built_and_cited_by_no_test": untested,
        "cited_by_a_done_task_but_defined_nowhere": sorted(done - every),
    }
    if "--json" in sys.argv[1:]:
        print(json.dumps(report, indent=2))
        return 0
    print(f"requirements defined in spec/: {report['requirements']}")
    print(f"built (cited by a done task): {report['built']}")
    print(f"built and cited by a test: {report['built_and_cited_by_a_test']}")
    print(f"built and cited by no test: {len(untested)}")
    if report["cited_by_a_done_task_but_defined_nowhere"]:
        print("cited by a done task but defined nowhere: "
              + ", ".join(f"WPM-R{n}" for n in report["cited_by_a_done_task_but_defined_nowhere"]))
    for n in untested:
        print(f"WPM-R{n}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
