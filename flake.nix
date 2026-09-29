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

    # NanoYalla, the Rocq kernel the certificates are checked against: the
    # `nanoyalla' directory of Click & coLLecT, pinned to a commit since it
    # has no release of its own; `export::rocq::NANOYALLA' names the
    # version those files declare.
    nanoyalla = {
      url = "github:ComputerAidedLL/click-and-collect/6b1c25ff30a1c1dbfccdc0c1cebc272a67ab0137";
      flake = false;
    };
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } (inputs.import-tree ./modules);
}
