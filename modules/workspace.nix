# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The cargo workspace through crane. The dependencies are built once
# (`buildDepsOnly', from Cargo.lock) and the CLI and every check start from
# them; `workspace' passes the shared arguments on to checks.nix. A C library
# a crate links goes into `buildInputs' here.
{
  perSystem =
    {
      config,
      craneLib,
      lib,
      pkgs,
      ...
    }:
    let
      # Manifests, the lock, *.rs and *.toml only: editing anything else
      # rebuilds nothing.
      src = craneLib.cleanCargoSource ../.;

      commonArgs = {
        inherit src;
        strictDeps = true;

        buildInputs = lib.optionals pkgs.stdenv.hostPlatform.isDarwin [ pkgs.libiconv ];
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
    in
    {
      _module.args.workspace = {
        inherit src commonArgs cargoArtifacts;
      };

      packages = {
        linlog-cli = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            inherit (craneLib.crateNameFromCargoToml { cargoToml = ../cli/Cargo.toml; }) pname version;
            cargoExtraArgs = "--locked --package linlog-cli";
            # The `test' check runs them, once, for the whole workspace.
            doCheck = false;
          }
        );

        default = config.packages.linlog-cli;
      };
    };
}
