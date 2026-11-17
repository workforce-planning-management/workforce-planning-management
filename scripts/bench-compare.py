#!/usr/bin/env python3
"""Fail when a Criterion benchmark is slower than its saved baseline (WPM-T527; BENCHMARKS.md, "Comparing two runs").

Criterion prints a change against a baseline but never fails on one. This reads the change Criterion writes for each
benchmark (`<criterion dir>/<group>/<id>/change/estimates.json`) and fails when the whole 95% interval of the mean's change
is above the threshold, so a change Criterion cannot tell from noise never fails. Run it on one machine, with nothing else
running; shared CI runners are too noisy for this, so CI does not run it.

    # on the commit to compare against
    cargo bench --bench rules -- --save-baseline before
    # on the change
    cargo bench --bench rules -- --baseline before
    python3 scripts/bench-compare.py <criterion dir>            # default: the service's target/criterion
    python3 scripts/bench-compare.py <criterion dir> --threshold 5

The criterion directory is `target/criterion` in the crate, or `$CARGO_TARGET_DIR/criterion` when that is set. Exit 0
when none is slower, 1 when one is, 2 when there is nothing to compare.
"""

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT = ROOT / "workforce-planning-management-service-with-rust" / "target" / "criterion"


def changes(criterion: Path):
    """Each benchmark's name and the mean's relative change: point estimate and 95% interval."""
    for path in sorted(criterion.glob("**/change/estimates.json")):
        mean = json.loads(path.read_text())["mean"]
        interval = mean["confidence_interval"]
        name = path.parent.parent.relative_to(criterion).as_posix()
        yield name, mean["point_estimate"], interval["lower_bound"], interval["upper_bound"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("criterion", nargs="?", type=Path, default=DEFAULT)
    parser.add_argument("--threshold", type=float, default=10.0, help="percent slower that fails (default 10)")
    args = parser.parse_args()

    rows = list(changes(args.criterion))
    if not rows:
        print(f"no change estimates under {args.criterion}: run `cargo bench -- --baseline <name>` first")
        return 2
    limit = args.threshold / 100
    slower = []
    width = max(len("benchmark"), *(len(name) for name, *_ in rows))
    print(f"{'benchmark':<{width}}  {'change':>8}  {'95% interval':>19}")
    for name, point, low, high in rows:
        mark = ""
        if low > limit:
            mark = "  SLOWER"
            slower.append(name)
        elif high < -limit:
            mark = "  faster"
        print(f"{name:<{width}}  {point:>+8.1%}  [{low:>+7.1%}, {high:>+7.1%}]{mark}")
    if slower:
        print(f"{len(slower)} of {len(rows)} benchmarks slower than +{args.threshold:g}%, the whole interval above it")
        return 1
    print(f"none of {len(rows)} benchmarks slower than +{args.threshold:g}%")
    return 0


if __name__ == "__main__":
    sys.exit(main())
