# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# `nix flake check', besides treefmt-nix's own `treefmt' check. cargo-deny's
# `advisories' needs the network, so it is not a check here: run
# `cargo deny check' in the shell.
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

        doc = craneLib.cargoDoc (
          withArtifacts
          // {
            env.RUSTDOCFLAGS = "--deny warnings";
          }
        );

        deny = craneLib.cargoDeny { inherit (workspace) src; };

        # Every combination of the core crate's features; the CLI enables all
        # of them and would never notice one that breaks alone.
        features = craneLib.mkCargoDerivation (
          withArtifacts
          // {
            pnameSuffix = "-features";
            nativeBuildInputs = [ pkgs.cargo-hack ];
            buildPhaseCargoCommand = "cargo hack check --feature-powerset --locked --package linlog";
            doInstallCargoArtifacts = false;
          }
        );

        deadnix = pkgs.runCommand "check-deadnix" { nativeBuildInputs = [ pkgs.deadnix ]; } ''
          cd ${nixSources}
          deadnix --fail .
          touch $out
        '';
      };
    };
}
