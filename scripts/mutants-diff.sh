#!/usr/bin/env bash
# Mutation-test the lines changed since a base commit (WPM-T526; spec/testing.md, "Mutation testing").
#
#   scripts/mutants-diff.sh                 # changes since origin/main, including uncommitted ones
#   scripts/mutants-diff.sh <commit>        # changes since <commit>
#
# For each crate with a mutants configuration (the service, whose `.cargo/mutants.toml` examines only the pure rules and
# `src/security.rs` with the lib's unit tests, and the matcher), it lists the mutants in the changed lines and tests them.
# A mutant no test catches, or one that hangs, fails the run. When there are more than the cap, one even slice of them is
# tested (`--shard 0/n`, round robin) and the run says so: a sample, so a change too big for one job is still checked
# without timing out. A full run of a crate is `cargo mutants` in its directory.
#
# **It mutates the checkout itself** (`--in-place`, one mutant at a time, each file put back after its test). The service
# cannot be tested in a copy, because its unit tests read files from `spec/` outside the crate; in place also reuses the
# crate's own build. A run killed hard can leave one file mutated: `git diff` shows it, and `git checkout -- <file>` undoes it.
#
# Settings: MUTANTS_CAP_SERVICE (default 15) and MUTANTS_CAP_MATCHER (default 40) mutants a crate; MUTANTS_OUT, where the
# reports go (default target/mutants-diff in each crate); MUTANTS_CRATES, the crate directories to run (default both).
set -euo pipefail

root=$(git rev-parse --show-toplevel)
base="${1:-origin/main}"
# A push that creates a branch has no "before"; a commit a shallow clone lacks is no base either. Fall back to the parent.
if [ -z "$base" ] || ! git -C "$root" cat-file -e "${base}^{commit}" 2>/dev/null; then
  echo "mutants: no base commit '${base}'; using HEAD~1"
  base=$(git -C "$root" rev-parse HEAD~1)
fi

status=0
for crate in ${MUTANTS_CRATES:-workforce-planning-management-service-with-rust workforce-planning-management-matcher-with-rust}; do
  case "$crate" in
    *service*) cap="${MUTANTS_CAP_SERVICE:-15}" ;;
    *) cap="${MUTANTS_CAP_MATCHER:-40}" ;;
  esac
  diff=$(mktemp)
  # Paths relative to the crate, as cargo-mutants reads them; the working tree, so uncommitted changes count locally.
  git -C "$root" diff --relative="$crate/" "$base" -- "$crate" >"$diff"
  cd "$root/$crate"
  if [ ! -s "$diff" ]; then
    echo "mutants: $crate: nothing changed"
    continue
  fi
  n=$(cargo mutants --list --in-diff "$diff" | wc -l | tr -d ' ')
  if [ "$n" -eq 0 ]; then
    echo "mutants: $crate: no mutants in the changed lines"
    continue
  fi
  args=(--in-diff "$diff" --output "${MUTANTS_OUT:-target/mutants-diff}")
  if [ "$n" -gt "$cap" ]; then
    shards=$(((n + cap - 1) / cap))
    echo "mutants: $crate: $n mutants in the changed lines, more than $cap; testing a sample of 1 in $shards"
    args+=(--shard "0/$shards" --sharding round-robin)
  else
    echo "mutants: $crate: $n mutants in the changed lines"
  fi
  args+=(--in-place)
  if ! cargo mutants "${args[@]}"; then
    status=1
    echo "mutants: $crate: a mutant survived or hung; see ${MUTANTS_OUT:-target/mutants-diff}/mutants.out/missed.txt"
    cat "${MUTANTS_OUT:-target/mutants-diff}/mutants.out/missed.txt" "${MUTANTS_OUT:-target/mutants-diff}/mutants.out/timeout.txt" 2>/dev/null || true
  fi
done
exit "$status"
