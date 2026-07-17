# Global Rules

For this repository, always:

1. Maintain `.codex/repo_overview.md` using the repo overview maintenance routine before starting any new PR and after finishing it.
2. Run `ci/local_check.sh` at the end of work and ensure it passes, or explain precisely why it cannot be made to pass as part of the PR.
3. Prefer existing Greentic repos/crates for shared interfaces, types, secrets, OAuth, messaging, events, and similar cross-cutting behavior instead of redefining them locally.

## Workflow for Every PR

1. Pre-PR sync:
   - Check out the target branch for the work, usually the default branch unless another branch is specified.
   - Fully refresh `.codex/repo_overview.md` so it reflects the current state before changes.
   - Show the updated overview if it changed meaningfully.

2. Implement the PR:
   - Apply requested code, tests, docs, and configuration changes.
   - Before adding core types, interfaces, or shared behavior, check whether a suitable Greentic crate already exists.
   - Use existing shared crates when they fit.
   - Document any clear reason for introducing a new shared concept locally.
   - Run appropriate build and test commands while working.

3. Post-PR sync:
   - Refresh `.codex/repo_overview.md` based on the updated codebase.
   - Run `ci/local_check.sh` from the repository root.
   - Fix failures caused by the current changes.
   - If failures are outside scope, capture the failing steps and key messages in the final summary.

## Behavioral Rules

- Do not ask for permission to run repo overview maintenance, run `ci/local_check.sh`, or reuse existing Greentic crates.
- Never leave `.codex/repo_overview.md` partially updated or obviously inconsistent.
- Never duplicate shared Greentic core types or interfaces without a documented reason.
- If build or test commands are unclear and cannot be inferred from repo files, ask a concise question.
