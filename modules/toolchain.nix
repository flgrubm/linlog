# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The toolchain rust-toolchain.toml names, shared by the devshell, rustfmt and
# crane, so the shell and the sandbox compile with the same rustc. `stable'
# resolves against the locked rust-overlay: it moves with flake.lock only.
#
# `mkRustBin' builds on the existing `pkgs' instead of re-importing nixpkgs
# with an overlay. crane takes the toolchain as a function of the package set
# so it can pick the right one when cross-compiling.
{ inputs, ... }:
{
  perSystem =
    { pkgs, ... }:
    let
      toolchainFor =
        p: (inputs.rust-overlay.lib.mkRustBin { } p).fromRustupToolchainFile ../rust-toolchain.toml;
    in
    {
      _module.args = {
        rustToolchain = toolchainFor pkgs;
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain toolchainFor;
      };
    };
}
