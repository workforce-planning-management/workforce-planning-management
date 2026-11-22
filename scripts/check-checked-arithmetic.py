#!/usr/bin/env python3
"""The searches RCA-ENF-5 and RCA-ENF-7 of spec/rust-checked-arithmetic/index.md ask CI to run, on production Rust code.

Production code is each file up to its first `#[cfg(test)]` line (tests sit at the end of a file). A hit fails the run.

  1. saturating_*, wrapping_* and overflowing_* (RCA-INT-4, RCA-INT-5).
  2. A checked_* result that is unwrapped, expected or defaulted (RCA-NONE-1).
  3. An input parse or lookup whose error is discarded and replaced (RCA-IN-2, RCA-IN-3), unless the line before
     says `// RCA-IN-3: absent only` and cites the declared default.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SERVICE = ROOT / "workforce-planning-management-service-with-rust"
DIRS = [SERVICE / "src", SERVICE / "crates" / "workforce-planning-management-domain" / "src"]

BANNED = re.compile(r"\b(saturating|wrapping|overflowing)_\w+")
UNWRAPPED = re.compile(r"checked_\w+\([^;]*?\)\s*\.(unwrap|expect|unwrap_unchecked|unwrap_or_default)\b", re.S)
PARSE = re.compile(r"\.parse(::<[^>]+>)?\(\)[^;]*?\.(ok\(\)|unwrap_or|unwrap_or_default|unwrap_or_else)\b", re.S)
LOOKUP = re.compile(r"(env::var|var_os|get|get_param|query)\([^;]*?\.ok\(\)[^;]*?\.(unwrap_or|unwrap_or_default|unwrap_or_else)\b", re.S)
MARK = "RCA-IN-3: absent only"


def production(path: Path) -> str:
    text = path.read_text()
    cut = text.find("#[cfg(test)]")
    return text if cut < 0 else text[:cut]


def line_of(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def main() -> int:
    problems = []
    for base in DIRS:
        for path in sorted(base.rglob("*.rs")):
            text = production(path)
            lines = text.split("\n")
            rel = path.relative_to(ROOT)
            for number, line in enumerate(lines, 1):
                code = line.split("//")[0]
                if BANNED.search(code):
                    problems.append(f"{rel}:{number}: RCA-INT-4/5: {line.strip()}")
            for pattern, why in ((UNWRAPPED, "RCA-NONE-1: a checked result is unwrapped"),):
                for m in pattern.finditer(text):
                    problems.append(f"{rel}:{line_of(text, m.start())}: {why}")
            for pattern in (PARSE, LOOKUP):
                for m in pattern.finditer(text):
                    if "\n}\n" in m.group(0):  # the match ran out of its function
                        continue
                    number = line_of(text, m.start())
                    nearby = "\n".join(lines[max(0, number - 3):number])
                    if MARK not in nearby:
                        problems.append(f"{rel}:{number}: RCA-IN-2: an input error is discarded and replaced")
    for problem in problems:
        print(problem)
    print(f"{len(problems)} checked-arithmetic problem(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
