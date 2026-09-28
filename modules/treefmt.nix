# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# `nix fmt' and the `treefmt' check. Markdown, zellij.kdl and
# .claude/settings.json (which Claude Code rewrites itself) are left alone.
{ inputs, ... }:
{
  imports = [ inputs.treefmt-nix.flakeModule ];

  perSystem =
    {
      config,
      lib,
      rustToolchain,
      ...
    }:
    {
      treefmt = {
        programs = {
          nixfmt.enable = true;

          # The toolchain's rustfmt, the one `cargo fmt' runs. treefmt passes
          # single files, so the edition has to be given explicitly.
          rustfmt = {
            enable = true;
            package = rustToolchain;
            inherit ((lib.importTOML ../Cargo.toml).workspace.package) edition;
          };

          # Configured by taplo.toml, which `taplo fmt' reads as well.
          taplo.enable = true;

          shfmt.enable = true;
          shellcheck.enable = true;
        };

        # treefmt walks the whole directory outside a VCS work tree.
        settings.global.excludes = [
          "target/**"
          ".direnv/**"
          "result*"
        ];
      };

      # `treefmt --stdin FILE' formats an editor buffer exactly as `nix fmt'
      # would; taplo is the same binary with the same taplo.toml.
      devshells.default.devshell.packages = [
        config.treefmt.build.wrapper
        config.treefmt.build.programs.taplo
      ];
    };
}
