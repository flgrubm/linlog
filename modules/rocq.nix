# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The Rocq certificates check: NanoYalla, the kernel the scripts are written
# for, is built from the flake input with Rocq and its standard library from
# nixpkgs, then every certificate the core tests pin in core/tests/snapshots
# and two the CLI writes are compiled against it. Rocq prints nothing for a
# script it accepts, so any output from a certificate fails the check; the
# kernel's own build may warn (it imports `From Coq', which Rocq 9
# deprecates), which only its exit status judges. `Qed' closes every
# lemma, so a script Rocq accepts is a proof the kernel checked, not one it
# admitted. The closure is Rocq's, about 1.2 GB, all from the binary cache.
{ inputs, ... }:
{
  perSystem =
    { config, pkgs, ... }:
    let
      snapshots = pkgs.lib.fileset.toSource {
        root = ../core/tests/snapshots;
        fileset = pkgs.lib.fileset.fileFilter (file: file.hasExt "v") ../core/tests/snapshots;
      };

      stdlib = pkgs.rocqPackages.stdlib;
    in
    {
      checks.rocq =
        pkgs.runCommand "check-rocq"
          {
            nativeBuildInputs = [
              config.packages.linlog-cli
              pkgs.rocq-core
              stdlib
            ];
            ROCQPATH = "${stdlib}/lib/coq/${pkgs.rocq-core.rocq-version}/user-contrib";
          }
          ''
            export HOME=$TMPDIR
            cp -r ${inputs.nanoyalla}/nanoyalla nanoyalla
            chmod -R u+w nanoyalla
            for file in nanoll macroll; do
              rocq compile -R nanoyalla NanoYalla nanoyalla/$file.v >kernel.log 2>&1 ||
                { cat kernel.log; exit 1; }
            done
            cp ${snapshots}/*.v .
            linlog prove --format rocq --standalone --output cli.v 'A * top |- A * (B + top)'
            linlog prove -i --format rocq --standalone --output cli_ill.v '!A, A -o B |- B * !A'
            for file in *.v; do
              rocq compile -R nanoyalla NanoYalla "$file" 2>&1 | tee log
              if [ -s log ]; then exit 1; fi
            done
            touch $out
          '';
    };
}
