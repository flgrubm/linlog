# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

{
  description = "linlog: A linear logic suite for all your needs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixpkgs-lib.url = "github:nix-community/nixpkgs.lib";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs-lib";
    };

    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane.url = "github:ipetkov/crane";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = inputs.nixpkgs.lib.systems.flakeExposed;
      debug = true;
      imports = with inputs; [
        devshell.flakeModule
        treefmt-nix.flakeModule
      ];

      perSystem =
        {
          self',
          pkgs,
          config,
          lib,
          system,
          ...
        }:
        let
          devshell_dependencies = with pkgs; [ zellij ];
          toolchain = pkgs.rust-bin.stable.latest.default;
          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain (_: toolchain);

          src = craneLib.cleanCargoSource ./.;

          commonArgs = {
            inherit src;
            strictDeps = true;
            buildInputs = [
              # Add additional build inputs here
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              pkgs.libiconv
            ];
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          individualCrateArgs = commonArgs // {
            inherit cargoArtifacts;
            inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
            # NB: we disable tests since we'll run them all via cargo-nextest
            doCheck = false;
          };

          fileSetForCrate =
            crate:
            lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                (craneLib.fileset.commonCargoSources ./core)
                (craneLib.fileset.commonCargoSources crate)
              ];
            };

          # linlog = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });

          linlog-cli = craneLib.buildPackage (
            individualCrateArgs
            // {
              pname = "linlog-cli";
              cargoExtraArgs = "-p linlog-cli";
              src = fileSetForCrate ./cli;
            }
          );

          linlog-workspace-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              inputs.rust-overlay.overlays.default
            ];
          };

          checks = {
            inherit linlog-cli;
            inherit linlog-workspace-clippy;
            linlog-crate-fmt = craneLib.cargoFmt {
              inherit src;
            };
            linlog-crate-toml-fmt = craneLib.taploFmt {
              src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
            };
          };

          packages = {
            default = linlog-cli;
            inherit linlog-cli;
          };

          devshells.default = {
            packages = devshell_dependencies ++ [
              toolchain
              pkgs.rust-analyzer
              pkgs.gcc
            ];
            env = [
              {
                name = "RUST_SRC_PATH";
                value = "${toolchain}/lib/rustlib/src/rust/library";
              }
            ];

            commands = [
              {
                name = "dev";
                help = "Enter `zellij` development environment";
                command = "zellij --layout zellij.kdl";
              }
            ];
          };

          treefmt = {
            programs = {
              nixfmt.enable = true;
              rustfmt.enable = true;
              taplo.enable = true;
            };
            projectRootFile = "./flake.nix";
          };
        };
    };
}
