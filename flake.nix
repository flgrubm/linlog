# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Dendritic flake-parts: import-tree loads every .nix file under modules/ as a
# flake-parts module (paths with a `_' segment are skipped), so this file only
# declares the inputs.
{
  description = "linlog: A linear logic suite for all your needs";

  inputs = {
    # The channel tarball: the same Hydra-tested revision as the GitHub
    # branch, without GitHub's rate limits. It still locks to a `rev'.
    nixpkgs.url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.zst";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    import-tree.url = "github:denful/import-tree";

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

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } (inputs.import-tree ./modules);
}
