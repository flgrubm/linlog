# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The LaTeX and Typst exports compile: every derivation the core tests pin
# in core/tests/snapshots, and a proof in each format as the CLI writes it,
# with pdfLaTeX from a minimal TeX Live and Typst with curryst from nixpkgs,
# offline. The curryst here is the version `linlog::export::typst::CURRYST'
# names; they change together.
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
        ps.standalone
      ]);

      typst = pkgs.typst.withPackages (ps: [ ps.curryst_0_6_0 ]);
    in
    {
      checks.export =
        pkgs.runCommand "check-export"
          {
            nativeBuildInputs = [
              config.packages.linlog-cli
              latex
              typst
            ];
          }
          ''
            export HOME=$TMPDIR
            cp ${snapshots}/* .
            linlog prove -i --format latex --standalone --output cli.tex '!A, A -o B |- B * !A'
            linlog prove --format typst --standalone --output cli.typ 'A & B, !C |- (B + A) * !C'
            for file in *.tex; do
              pdflatex -interaction=nonstopmode -halt-on-error "$file" >/dev/null ||
                { cat "''${file%.tex}.log"; exit 1; }
            done
            for file in *.typ; do
              typst compile "$file"
            done
            touch $out
          '';
    };
}
