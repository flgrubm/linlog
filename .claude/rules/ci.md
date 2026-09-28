---
paths:
  - ".github/**"
---

# CI: the GitHub workflows

Loaded when a file under `.github/` is read.

- **Every tool comes from the flake.** A job installs Nix and runs
  `nix flake check`, `nix build` or `nix develop --command …`. No
  `setup-rust`, `apt` or `cargo install`: they would check with other versions
  than the devshell does. A new check goes into the flake
  (`modules/checks.nix`), not into a workflow step.
- **Third-party actions are pinned to a full commit SHA**, with the release as
  a comment: `uses: owner/repo@<sha> # vX.Y.Z`. Resolve a tag with
  `gh api repos/<owner>/<repo>/commits/<tag> --jq .sha`, and read that
  commit's `action.yml` for its inputs. Dependabot proposes the bumps, weekly,
  as one pull request. The pin of `cachix/install-nix-action` also fixes the
  Nix version CI runs.
- **Least privilege**: `permissions: contents: read` at the top, anything more
  per job; `actions/checkout` with `persist-credentials: false`.
- `install-nix-action` gets `github_access_token: ${{ github.token }}`: without
  it, fetching the flake's `github:` inputs from shared runners hits the
  anonymous rate limit.
- **No workflow runs locally.** `nix flake check` runs actionlint, with
  shellcheck on the `run:` scripts, over `.github/workflows/`: it catches
  syntax, unknown runner labels, bad expressions and shell mistakes, not
  behaviour on GitHub. Read run results with `gh run list` and
  `gh run view --log-failed`.
