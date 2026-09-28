# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Not `lib.systems.flakeExposed': that adds platforms nobody builds linlog on.
# x86_64-darwin is gone from nixpkgs.
{
  systems = [
    "x86_64-linux"
    "aarch64-linux"
    "aarch64-darwin"
  ];
}
