# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# `nix flake check', besides treefmt-nix's own `treefmt' check. cargo-deny's
# `advisories' needs the network, so it is not a check here: run
# `cargo deny check' in the shell (CI runs it on every push and weekly).
{
  perSystem =
    {
      config,
      craneLib,
      lib,
      pkgs,
      workspace,
      ...
    }:
    let
      withArtifacts = workspace.commonArgs // {
        inherit (workspace) cargoArtifacts;
      };

      nixSources = lib.fileset.toSource {
        root = ../.;
        fileset = lib.fileset.fileFilter (file: file.hasExt "nix") ../.;
      };

      workflowSources = lib.fileset.toSource {
        root = ../.;
        fileset = ../.github/workflows;
      };
    in
    {
      checks = {
        build = config.packages.linlog-cli;

        clippy = craneLib.cargoClippy (
          withArtifacts
          // {
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          }
        );

        # `cargo test', not nextest: nextest skips doc tests.
        test = craneLib.cargoTest withArtifacts;

        # The tests once more with the assertions a development build has:
        # crane tests in the release profile, where `debug_assert!' is
        # nothing, and the engines state their invariants in that macro (a
        # proof checked where it is made, a zone that must be empty). The
        # doc tests ran above.
        test-debug-assertions = craneLib.cargoTest (
          withArtifacts
          // {
            pnameSuffix = "-test-debug-assertions";
            CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS = "true";
            CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS = "true";
            cargoTestExtraArgs = "--all-targets";
          }
        );

        doc = config.packages.doc;

        deny = craneLib.cargoDeny { inherit (workspace) src; };

        # The core crate's features one at a time and in pairs, with all of
        # them and with none; the CLI enables all of them and would never
        # notice one that breaks alone. The full powerset doubles with every
        # feature, and a breakage needs more than two features rarely.
        features = craneLib.mkCargoDerivation (
          withArtifacts
          // {
            pnameSuffix = "-features";
            nativeBuildInputs = [ pkgs.cargo-hack ];
            buildPhaseCargoCommand = ''
              cargo hack check --each-feature --locked --package linlog
              cargo hack check --feature-powerset --depth 2 --locked --package linlog
            '';
            doInstallCargoArtifacts = false;
          }
        );

        deadnix = pkgs.runCommand "check-deadnix" { nativeBuildInputs = [ pkgs.deadnix ]; } ''
          cd ${nixSources}
          deadnix --fail .
          touch $out
        '';

        # The GitHub workflows, which no local run exercises otherwise;
        # nixpkgs' actionlint runs shellcheck on their `run' scripts.
        actionlint = pkgs.runCommand "check-actionlint" { nativeBuildInputs = [ pkgs.actionlint ]; } ''
          cd ${workflowSources}
          actionlint .github/workflows/*
          touch $out
        '';
      };
    };
}
