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
      # Manifests, the lock, *.rs and *.toml, and the export snapshots the
      # core tests compare with: editing anything else rebuilds nothing.
      src = lib.fileset.toSource {
        root = ../.;
        fileset = lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ../.)
          ../core/tests/snapshots
        ];
      };

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
            # The binary is `linlog', not the package name `nix run' assumes.
            meta.mainProgram = "linlog";
          }
        );

        # The rustdoc site the Docs workflow publishes, which the `doc' check
        # also builds: both crates, `linlog' and `linlog_cli', in one tree.
        # rustdoc writes no top-level index, so one redirects to the core
        # crate.
        doc = craneLib.cargoDoc (
          commonArgs
          // {
            inherit cargoArtifacts;
            env.RUSTDOCFLAGS = "--deny warnings";
            postInstall = ''
              cat > $out/share/doc/index.html <<'EOF'
              <!DOCTYPE html>
              <meta charset="utf-8">
              <title>linlog</title>
              <meta http-equiv="refresh" content="0; url=linlog/">
              <a href="linlog/">linlog</a>
              EOF
            '';
          }
        );

        default = config.packages.linlog-cli;
      };
    };
}
