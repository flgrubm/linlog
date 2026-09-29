#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Regenerates the baseline: the CSV files in bench/results/ and
# bench/RESULTS.md, their tables. Every family sequentially with its
# default engine and with every engine that applies, the net engine's test
# period, the intuitionistic mode of the families that have one, the whole
# LLTP library in both modes, then the thread counts 2, 4 and every core on
# the hard families, a portfolio on the slow LLTP problems, and the
# intuitionistic LLTP problems again with a copy bound of 10. About five
# hours on sixteen cores.
#
#   nix build .#lltp -o bench/lltp    # once: the LLTP library
#   rm -rf bench/results               # for a fresh baseline
#   systemd-run --user --unit=linlog-baseline --same-dir -p MemoryMax=24G \
#     -p MemorySwapMax=0 -p OOMPolicy=continue -p LimitCORE=0 \
#     --setenv=PATH="$PATH" bench/baseline.sh
#   journalctl --user -fu linlog-baseline
#
# (from the devshell: a unit of its own survives the terminal, and its
# memory limit keeps a runaway from taking the machine down; with
# `OOMPolicy=continue' the kernel kills the runaway alone, whose run is
# then a crash row, instead of systemd stopping the whole unit).
#
# Every run appends to its CSV file and skips what the file already has
# (`--append --resume'), so running the script again after an interruption
# finishes the baseline instead of starting over.
#
# The sequential runs go in four streams at once, each pinned to a core of
# its own with taskset; `cores' names four performance cores of the machine
# the baseline was taken on (an Intel Core Ultra X9 388H: CPUs 0 to 3 are
# its performance cores), so adjust it for another. The parallel runs come
# after them, with the machine to themselves.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release --locked --package linlog-bench
bench=${CARGO_TARGET_DIR:-target}/release/linlog-bench
lltp=bench/lltp
out=bench/results
cores=(1 0 2 3)
mkdir -p "$out"

# run CPU NAME ARGS...: one `run` into NAME.csv, pinned to CPU unless it is
# `-`. Every process gets 16 GiB of address space, so that a search whose
# memory grows without bound (the additive path's memo has no cap) fails
# its own run instead of swapping the machine.
run() {
  local cpu=$1 name=$2
  shift 2
  local pin=(prlimit --as=$((16 << 30)))
  if [ "$cpu" != - ]; then
    pin+=(taskset -c "$cpu")
  fi
  "${pin[@]}" "$bench" run "$@" --output "$out/$name.csv" --append --resume 2>>"$out/$name.log"
}

repeat=(--repeat 3 --repeat-under 2)
mll=(--family "partition-yes=4,5,6" --family "partition-no=3,4" --family 3-partition-mll-yes
  --family 3-partition-mll-no --family wide-m1 --family wide-m2 --family "wide-m3=12,24,30"
  --family "wide-m4=12,24,28")

run "${cores[0]}" families --all-families --timeout 300 "${repeat[@]}" &
(
  run "${cores[1]}" engines "${mll[@]}" --family additive --problems bench/problems/slow-tests.txt \
    --engines focus,net --timeout 60 "${repeat[@]}"
  for period in 1 2 8 16; do
    run "${cores[1]}" period-$period --family wide-m1=256,2048 --family wide-m2=256,2048 \
      --family 3-partition-mll-no=4,5 --family partition-yes=4 --family partition-no=3,4 \
      --engines net --test-period $period --timeout 60 "${repeat[@]}"
  done
  run "${cores[1]}" intuitionistic --family counter --family counter-over --family chain \
    --family 3-partition-yes --family 3-partition-no=4 --modes intuitionistic --timeout 300 \
    "${repeat[@]}"
) &
run "${cores[2]}" lltp-intuitionistic --lltp "$lltp/ILL" --timeout 5 &
run "${cores[3]}" lltp-classical --lltp "$lltp/CLL" --lltp "$lltp/ILL" --modes classical \
  --timeout 5 &
wait

# The parallel runs, alone on the machine.
run - parallel --family 3-partition-yes --family 3-partition-no --family partition-yes=5,6,7 \
  --family partition-no=4,5 --family qbf=16,20,24 --family mix=8,9,10,11 --family counter \
  --family counter-over --family wide-m3=24,30,36 --family wide-m4=24,28,32 \
  --jobs 2,4,all --timeout 120 "${repeat[@]}"
run - parallel-net --family 3-partition-mll-no=4,5,6 --problems bench/problems/slow-tests.txt \
  --only partition-table --engines net --jobs 2,4,all --timeout 60 "${repeat[@]}"
# The LLTP problems decided in a twentieth of a second or more on one
# thread, on every core with and without the portfolio.
slow=$(awk -F, 'NR > 1 && $15 ~ /proved|unprovable/ && $22 >= 50 { print $5 }' \
  "$out/lltp-intuitionistic.csv" | sort -u | paste -sd,)
if [ -n "$slow" ]; then
  run - portfolio --lltp "$lltp/ILL" --only "$slow" --jobs all --timeout 5
  run - portfolio-on --lltp "$lltp/ILL" --only "$slow" --jobs all --portfolio --timeout 5
fi

# The copy bound of 3 ends most of the library's searches: the
# intuitionistic problems again with a bound of 10.
run "${cores[2]}" lltp-copies-10 --lltp "$lltp/ILL" --copies 10 --timeout 5

{
  cat <<EOF
# Benchmark results

The baseline of $(date +%Y-%m-%d) on an $(lscpu | sed -n 's/^Model name: *//p')
($(nproc) cores, $(free -g | awk '/^Mem:/ { print $2 }') GB), release build, from
\`bench/baseline.sh\`, which regenerates this file and the CSV files beside
it (\`bench/results/\`, one row per run). The sequential runs went in four
streams at once, each pinned to a performance core of its own; the
parallel runs had the machine to themselves. A time is the median of up to
three runs; \`✓\` is proved, \`✗\` refuted, \`?\` unknown (\`bound\`: the copy
bound, \`wide\`: a context too wide to split, \`depth\`: the recursion
limit), \`>\` a time limit reached, \`MISMATCH\` a verdict against the
problem's known one (for LLTP, the one its header claims). \`linlog-bench
summary bench/results/*.csv\` prints the tables again.

EOF
  "$bench" summary "$out"/*.csv
} >bench/RESULTS.md
