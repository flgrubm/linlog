# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The benchmark harness. `linlog-bench' is the package; the `bench' check
# runs it on the smallest instance of every generated family and on the
# problem file with a short time limit, and fails on a verdict that
# contradicts the one a problem is known to have: it keeps the harness
# building and running, while its timings on a build machine mean nothing.
# `lltp' is the LLTP problem library at a pinned commit with its Petri-net
# archives unpacked, for `linlog-bench run --lltp': it is GPL-3.0, so it is
# fetched on demand rather than kept in this repository, and at 1.1 GB it
# is no input of any check.
{
  perSystem =
    {
      craneLib,
      pkgs,
      workspace,
      ...
    }:
    let
      linlog-bench = craneLib.buildPackage (
        workspace.commonArgs
        // {
          inherit (workspace) cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml { cargoToml = ../bench/Cargo.toml; }) pname version;
          cargoExtraArgs = "--locked --package linlog-bench";
          # The `test' check runs them, once, for the whole workspace.
          doCheck = false;
          meta.mainProgram = "linlog-bench";
        }
      );
    in
    {
      packages = {
        inherit linlog-bench;

        lltp = pkgs.fetchFromGitHub {
          owner = "meta-logic";
          repo = "lltp";
          rev = "e0394fb8e9f6ad5127c92460c3f0936ccf64b693";
          hash = "sha256-9S6YdIlakpJkYOcUg1IkzbaOW2LLSk7GmmJJYSpomFs=";
          # The library's one malformed file has a tab where a closing
          # parenthesis belongs, the only tab in the library; repaired, the
          # problem has the size its neighbours in the family predict (each
          # twice the previous and 34 occurrences).
          postFetch = ''
            cat $out/ILL/petri-nets/MCC.tar.gz.* | tar -xz -C $out/ILL/petri-nets
            rm $out/ILL/petri-nets/MCC.tar.gz.*
            syj=$out/ILL/ILLTP-SYJ-01/SYJ206+1.018.p
            tr '\t' ')' <"$syj" >"$syj.repaired"
            mv "$syj.repaired" "$syj"
          '';
        };
      };

      checks.bench =
        pkgs.runCommand "check-bench"
          {
            nativeBuildInputs = [ linlog-bench ];
          }
          ''
            families=$(linlog-bench families | sed -E 's/^([^ ]+) \(([0-9]+).*/--family \1=\2/')
            # shellcheck disable=SC2086
            linlog-bench run $families --problems ${../bench/problems}/slow-tests.txt \
              --timeout 2 --output runs.csv 2> progress.log
            linlog-bench summary runs.csv > summary.md
            if grep -q MISMATCH summary.md; then
              cat summary.md
              exit 1
            fi
            touch $out
          '';
    };
}
