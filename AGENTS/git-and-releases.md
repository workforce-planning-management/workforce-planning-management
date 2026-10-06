# Git and releases

## The repository rules (from `CONTRIBUTING.md`, `GOVERNANCE.md`, `AI_STATEMENT.md`)

- **Branch per change**: branch from `main`, merge back with `--no-ff`; never
  squash, never commit straight to `main`.
- **Three-part change**: spec + code + test, with the owning subproject's
  `CHANGELOG.md` updated in the same change.
- **Green gate** before merging: see
  [testing-and-verification](testing-and-verification.md).
- **AI tooling** may merge a pull request once it clears the explicit checklist in
  `AI_STATEMENT.md` §6, and may judge an already-merged version bump ready and
  publish it (`cargo publish` for the service, `npm publish` for the front-end)
  per §7 — never without a maintainer-decided *what* behind it. A contribution
  with AI-generated content says so in the pull-request description.

## What has actually happened in sessions so far

Sessions have worked **on `main`**, one commit per task, at the maintainer's
direction, with the maintainer asking for each push. If you are unsure whether
the maintainer wants a branch or a direct commit, **ask** — the written rule is
the branch.

## Commits

- One logical change per commit; the message says what and why. Reference the
  task id (`… (WPM-T89)`).
- Add the attribution trailer your session specifies
  (`Co-Authored-By: …`); it is attribution, not disclosure
  (`AI_STATEMENT.md` §10).
- Stage deliberately; do not commit scratch files, `test-results/`, build output
  or machine-specific paths. A `.sota/` scan record is committed on purpose.

## Pushing

- `origin` has one fetch URL (GitHub) and **three push URLs** (Codeberg, GitHub,
  GitLab): a single `git push` updates all three.
- **Push only when the maintainer asks.** Show what is local-only (`git log
  origin/main..`) when asking.
- Never force-push, and do not rewrite history that has been pushed.

## Releases

Version bumps and publishing follow `AI_STATEMENT.md` §7: the bump is already on
`main`, the gates are green, the changelog is accurate, nothing else is pending.
Publishing to a registry is irreversible — confirm first.
