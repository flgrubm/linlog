# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The exports compile and render: every derivation and net the core tests
# pin in core/tests/snapshots, and a proof in each format as the CLI writes
# it, with pdfLaTeX from a minimal TeX Live, Typst with curryst from nixpkgs
# and resvg, offline. The curryst here is the version
# `linlog::export::typst::CURRYST' names; they change together. Typst and
# resvg see Euler Math and no system font, and anything either prints fails
# the check, so a font they cannot find is not hidden by a fallback.
{
  perSystem =
    {
      config,
      lib,
      pkgs,
      ...
    }:
    let
      snapshots = lib.fileset.toSource {
        root = ../core/tests/snapshots;
        fileset = ../core/tests/snapshots;
      };

      latex = pkgs.texliveBasic.withPackages (ps: [
        ps.amsfonts
        ps.cmll
        ps.ebproof
        ps.eulervm
        ps.standalone
      ]);

      typst = pkgs.typst.withPackages (ps: [ ps.curryst_0_6_0 ]);

      eulerMath = "${pkgs.texlivePackages.euler-math.tex}/fonts/opentype/public/euler-math";
    in
    {
      checks.export =
        pkgs.runCommand "check-export"
          {
            nativeBuildInputs = [
              config.packages.linlog-cli
              latex
              typst
              pkgs.resvg
            ];
          }
          ''
            export HOME=$TMPDIR
            cp ${snapshots}/* .
            linlog prove -i --format latex --standalone --output cli.tex '!A, A -o B |- B * !A'
            linlog prove --format typst --standalone --output cli.typ 'A & B, !C |- (B + A) * !C'
            linlog prove -i --format svg --output cli.svg '!A, A -o B |- B * !A'
            linlog prove --format net-svg --output cli-net.svg 'A -o B, B -o C |- A -o C'
            for file in *.tex; do
              pdflatex -interaction=nonstopmode -halt-on-error "$file" >/dev/null ||
                { cat "''${file%.tex}.log"; exit 1; }
            done
            for file in *.typ; do
              typst compile --ignore-system-fonts --font-path ${eulerMath} "$file" 2>log
              if [ -s log ]; then cat log; exit 1; fi
            done
            for file in *.svg; do
              resvg --skip-system-fonts --use-fonts-dir ${eulerMath} "$file" "''${file%.svg}.png" 2>log
              if [ -s log ]; then cat log; exit 1; fi
            done
            touch $out
          '';
    };
}
