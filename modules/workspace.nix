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
            # The binary is `linlog', not the package name `nix run' assumes.
            meta.mainProgram = "linlog";
          }
        );

        # The rustdoc site the Docs workflow publishes, which the `doc' check
        # also builds. The library crate and the CLI's binary are both named
        # `linlog', so one `cargo doc' would write both into doc/linlog/ and
        # keep whichever came last: the CLI is documented first and its site
        # moved aside, then the library, and the two are installed as
        # share/doc (the library) and share/doc/cli. rustdoc writes no
        # top-level index, so a landing page links both.
        doc = craneLib.mkCargoDerivation (
          commonArgs
          // {
            inherit cargoArtifacts;
            pname = "linlog";
            version = (craneLib.crateNameFromCargoToml { cargoToml = ../core/Cargo.toml; }).version;
            pnameSuffix = "-doc";
            env.RUSTDOCFLAGS = "--deny warnings";
            doInstallCargoArtifacts = false;
            buildPhaseCargoCommand = ''
              cargoWithProfile doc --locked --no-deps --package linlog-cli
              mv target/doc target/cli-doc
              cargoWithProfile doc --locked --no-deps --package linlog
            '';
            installPhaseCommand = ''
              mkdir -p $out/share
              mv target/doc $out/share/doc
              mv target/cli-doc $out/share/doc/cli
              cat > $out/share/doc/index.html <<'EOF'
              <!DOCTYPE html>
              <meta charset="utf-8">
              <title>linlog</title>
              <h1>linlog</h1>
              <ul>
                <li><a href="linlog/">The library <code>linlog</code></a></li>
                <li><a href="cli/linlog/">The command line program <code>linlog</code></a></li>
              </ul>
              EOF
            '';
          }
        );

        default = config.packages.linlog-cli;
      };
    };
}
