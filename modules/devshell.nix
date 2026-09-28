# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The development shell; `menu' lists its commands. Their names avoid shell
# builtins, which win over PATH (`test' in bash and zsh, `run' and `watch' in
# nushell).
{ inputs, ... }:
{
  imports = [ inputs.devshell.flakeModule ];

  perSystem =
    { pkgs, rustToolchain, ... }:
    {
      devshells.default = {
        devshell.packages = [
          rustToolchain
          # rustc links through `cc'; a devshell, unlike mkShell, has none.
          pkgs.stdenv.cc
          pkgs.cargo-hack
          pkgs.cargo-deny
          pkgs.bacon
          pkgs.zellij
        ];

        commands = [
          {
            name = "check";
            help = "Run every flake check: build, clippy, tests, rustdoc, cargo-deny, cargo-hack, deadnix, actionlint, formatting";
            command = "nix flake check";
          }
          {
            name = "tests";
            help = "Run the workspace's tests (`cargo test --workspace`)";
            command = ''cargo test --workspace "$@"'';
          }
          {
            name = "launch";
            help = "Build and run the CLI; arguments are passed on";
            command = ''cargo run --package linlog-cli -- "$@"'';
          }
          {
            name = "live";
            help = "Re-check on every save with bacon (`live clippy`, `live test` for other jobs)";
            command = ''bacon "$@"'';
          }
          {
            name = "dev";
            help = "Enter the zellij development layout";
            command = ''zellij --layout "$PRJ_ROOT/zellij.kdl"'';
          }
          {
            name = "up";
            help = "Update the flake inputs and commit flake.lock with jj (unverified; /update-deps verifies first)";
            # Not `nix flake update --commit-lock-file': that is a git commit.
            command = ''
              set -eu

              # Keep the bump out of whatever work @ already holds.
              isolated=0
              if [ -n "$(jj diff -r @ --name-only)" ] || \
                 [ -n "$(jj log --no-graph -r @ -T description)" ]; then
                jj new
                isolated=1
              fi

              err=$(mktemp)
              trap 'rm -f "$err"' EXIT

              if ! nix flake update 2>"$err"; then
                cat "$err" >&2
                if [ "$isolated" = 1 ]; then jj abandon; fi
                exit 1
              fi
              cat "$err" >&2

              if [ -z "$(jj diff -r @ --name-only)" ]; then
                echo "flake.lock is already up to date; nothing to commit."
                if [ "$isolated" = 1 ]; then jj abandon; fi
                exit 0
              fi

              # nix's bullet list on stderr is the body --commit-lock-file writes.
              updates=$(awk '/^•/{p=1} p' "$err")
              if [ -n "$updates" ]; then
                msg="flake.lock: Update

              Flake lock file updates:

              $updates"
              else
                msg="flake.lock: Update"
              fi

              jj commit -m "$msg"
            '';
          }
        ];
      };
    };
}
