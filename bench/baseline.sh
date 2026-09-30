#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Regenerates the baseline: the CSV files in bench/results/ and
# bench/RESULTS.md, their tables. Every family sequentially with its
# default engine and with every engine that applies, the net engine's test
# period, the intuitionistic mode of the families that have one, the whole
# LLTP library in both modes; the largest sizes with 20 minutes each, the
# LLTP problems that ended at the copy bound or the recursion limit with
# those raised; then the thread counts 2, 4, 8 and every core on the hard
# families, and every core with and without the portfolio on the LLTP
# problems not decided at once. About eight hours on sixteen cores, on an
# otherwise idle machine.
#
#   nix build .#lltp -o bench/lltp          # once: the LLTP library
#   bench/baseline.sh --detach --fresh       # from the devshell
#   journalctl --user -fu linlog-baseline    # progress every ten minutes
#
# --detach runs the script as the systemd user unit `linlog-baseline',
# which survives the terminal: 24 GiB of memory and no swap for all of it,
# the kernel's OOM killer taking the runaway process alone
# (`OOMPolicy=continue', its run a crash row) rather than systemd stopping
# the unit, no core dumps, and its own target directory (target/baseline)
# so that builds in the checkout do not touch the binary it runs.
# `systemctl --user stop linlog-baseline' stops it. --fresh deletes
# bench/results first; without it every run appends to its CSV file and
# skips what the file already has (`--append --resume'), so running the
# script again after an interruption finishes the baseline. The script
# refuses to start on a busy machine (a load average above 1) or with a
# scheduled job due within nine hours (`systemctl list-timers': nix-gc at
# midnight, nix-optimise before four, backups) unless given --force: the
# timings are only worth what the machine's idleness is, and RESULTS.md
# records the load it started with and the jobs that ran meanwhile. Stop
# such timers for the night (`sudo systemctl stop nix-gc.timer', start them
# again afterwards) or start after them.
#
# The sequential runs go in four streams at once, each pinned to a core of
# its own with taskset; `cores' names four performance cores of the machine
# the baseline was taken on (an Intel Core Ultra X9 388H: CPUs 0 to 3 are
# its performance cores), so adjust it for another. The parallel runs come
# after them, with the machine to themselves.
set -euo pipefail
cd "$(dirname "$0")/.."
lltp=bench/lltp
out=bench/results
cores=(1 0 2 3)

detach=false fresh=false force=false
for arg; do
  case $arg in
  --detach) detach=true ;;
  --fresh) fresh=true ;;
  --force) force=true ;;
  *)
    echo "usage: bench/baseline.sh [--detach] [--fresh] [--force]" >&2
    exit 2
    ;;
  esac
done

# The processes other than the benchmark's that use a CPU now, by the
# second of two samples of top.
others() {
  top -b -n 2 -d 2 -o %CPU -w 200 | awk '
    /^top -/ { frame++ }
    frame == 2 && $1 ~ /^[0-9]+$/ && $9 + 0 >= 1 && $12 !~ /^(linlog-bench|top)$/ {
      printf "%s %s%%, ", $12, $9
    }' | head -c 300
}

# The system and user timers that fire between two times (seconds since
# the epoch), by the property given (NextElapseUSecRealtime for the next
# firing, LastTriggerUSec for the last), as "name hh:mm, …"; those that do
# next to nothing (log rotation, firmware metadata, tmpfiles) are left out.
timers() {
  local scope unit t
  for scope in --system --user; do
    systemctl "$scope" list-timers --all --no-legend | awk '{ print $(NF - 1) }' |
      while read -r unit; do
        case $unit in logrotate.timer | fwupd-refresh.timer | systemd-tmpfiles-clean.timer) continue ;; esac
        t=$(systemctl "$scope" show "$unit" -p "$3" --value --timestamp=unix)
        t=${t#@}
        if [ -n "$t" ] && [ "$t" -ge "$1" ] && [ "$t" -le "$2" ]; then
          printf '%s %s, ' "${unit%.timer}" "$(date -d "@$t" +%H:%M)"
        fi
      done
  done
}

read -r load _ </proc/loadavg
if ! $force && awk -v l="$load" 'BEGIN { exit !(l > 1) }'; then
  echo "the machine is busy (load average $load; $(others)); start when it is idle, or pass --force" >&2
  exit 1
fi
# The whole baseline takes about eight hours; a scheduled job in that
# window (nix-gc at midnight, nix-optimise before four, a backup) competes
# with it for a minute or so of CPU and disk.
now=$(date +%s)
due=$(timers "$now" $((now + 9 * 3600)) NextElapseUSecRealtime)
if ! $force && [ -n "$due" ]; then
  echo "timers fire during the next nine hours: ${due%, }; start after them, stop them for the night, or pass --force" >&2
  exit 1
fi
if $fresh; then
  rm -rf "$out"
fi
if $detach; then
  systemctl --user reset-failed linlog-baseline 2>/dev/null || true
  systemd-run --user --unit=linlog-baseline --same-dir -p MemoryMax=24G -p MemorySwapMax=0 \
    -p OOMPolicy=continue -p LimitCORE=0 --setenv=PATH="$PATH" \
    --setenv=CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target/baseline}" \
    "$PWD/bench/baseline.sh" --force
  echo "follow it with: journalctl --user -fu linlog-baseline"
  exit
fi

echo "started $(date '+%Y-%m-%d %H:%M'), load average $load, timers due: ${due:-none}"
cargo build --release --locked --package linlog-bench
bench=${CARGO_TARGET_DIR:-target}/release/linlog-bench
mkdir -p "$out"

# Every ten minutes: the last progress line of every run (with its estimate
# of the time left), the load and whatever else uses a CPU.
progress() {
  while sleep 600; do
    echo "== $(date +%H:%M), $((SECONDS / 60)) min in, load $(cut -d' ' -f1-3 /proc/loadavg); others: $(others)"
    for log in "$out"/*.log; do
      local line
      line=$(grep -a '^\[' "$log" | tail -n 1 || true)
      if [ -n "$line" ]; then
        echo "   $(basename "$log" .log): ${line:0:150}"
      fi
    done
  done
}
progress &
watcher=$!
trap 'kill $watcher 2>/dev/null || true' EXIT

# run CPU NAME ARGS...: one `run` into NAME.csv, pinned to CPU unless it is
# `-`. Every process gets 12 GiB of address space, so that a search whose
# memory grows without bound (the additive path's memo has no cap) or the
# parse of one of the library's largest files fails its own run; two such
# processes at once stay within the unit's 24 GiB.
run() {
  local cpu=$1 name=$2
  shift 2
  local pin=(prlimit --as=$((12 << 30)))
  if [ "$cpu" != - ]; then
    pin+=(taskset -c "$cpu")
  fi
  echo "$(date +%H:%M) $name"
  "${pin[@]}" "$bench" run "$@" --output "$out/$name.csv" --append --resume 2>>"$out/$name.log"
}

repeat=(--repeat 3 --repeat-under 2)
mll=(--family "partition-yes=4,5,6" --family "partition-no=3,4" --family 3-partition-mll-yes
  --family 3-partition-mll-no --family wide-m1 --family wide-m2 --family "wide-m3=12,24,30"
  --family "wide-m4=12,24,28")

# Stage 1, sequential, four streams: every family, the engines against each
# other, the net engine's test period, the intuitionistic mode, the whole
# LLTP library in both modes. About an hour and forty minutes.
streams=()
run "${cores[0]}" families --all-families --timeout 300 "${repeat[@]}" &
streams+=($!)
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
streams+=($!)
run "${cores[2]}" lltp-intuitionistic --lltp "$lltp/ILL" --timeout 5 &
streams+=($!)
# In reverse, so that the two passes parse the library's largest files
# (hundreds of megabytes each) at different times.
run "${cores[3]}" lltp-classical --lltp "$lltp/CLL" --lltp "$lltp/ILL" --modes classical \
  --reverse --timeout 5 &
streams+=($!)
wait "${streams[@]}"

# The intuitionistic LLTP problems whose first pass ended with the reason
# given (or, for `slow', timed out, was killed or took 50 ms or more), as a
# list for --only: the problems a knob can change.
ended() {
  awk -F, -v reason="$1" 'NR > 1 && ($16 == reason || (reason == "slow" &&
    ($16 ~ /^(timeout|killed)$/ || ($15 ~ /proved|unprovable/ && $22 >= 50)))) { print $5 }' \
    "$out/lltp-intuitionistic.csv" | sort -u | paste -sd,
}

# Stage 2, sequential, four streams: the largest instances that time out at
# 300 s, once each with 20 minutes, for the times the performance pass has
# to beat; and the LLTP problems that ended at the copy bound or at the
# recursion limit, again with a bound of 10 and a limit of 16384. About an
# hour and a half.
streams=()
run "${cores[0]}" long-1 --family 3-partition-no=5 --family partition-no=5 --family partition-yes=7 \
  --family counter=16 --timeout 1200 &
streams+=($!)
run "${cores[1]}" long-2 --family mix=11 --family wide-m3=36 --family wide-m4=36 \
  --family qbf=24 --only "qbf/24#0,mix,wide" --timeout 1200 &
streams+=($!)
run "${cores[2]}" lltp-copies-10 --lltp "$lltp/ILL" --only "$(ended copy_bound)" --copies 10 \
  --timeout 5 &
streams+=($!)
run "${cores[3]}" lltp-recursion --lltp "$lltp/ILL" --only "$(ended recursion_limit)" \
  --recursion-limit 16384 --timeout 5 &
streams+=($!)
wait "${streams[@]}"

# Stage 3, alone on the machine: the hard families on 2, 4, 8 and every
# core, the net engine's cubes, and the LLTP problems that are not decided
# at once on every core, with and without the portfolio (every core is the
# command's default). About four and a half hours.
run - parallel --family 3-partition-yes --family 3-partition-no --family partition-yes=5,6,7 \
  --family partition-no=4,5 --family qbf=16,20,24 --family mix=8,9,10,11 --family counter \
  --family counter-over --family wide-m3=24,30,36 --family wide-m4=28,32,36 \
  --jobs 2,4,8,all --timeout 120 "${repeat[@]}"
run - parallel-net --family 3-partition-mll-no=4,5,6 --problems bench/problems/slow-tests.txt \
  --only partition-table --engines net --jobs 2,4,8,all --timeout 60 "${repeat[@]}"
slow=$(ended slow)
run - lltp-all-cores --lltp "$lltp/ILL" --only "$slow" --jobs all --timeout 5
run - lltp-portfolio --lltp "$lltp/ILL" --only "$slow" --jobs all --portfolio --timeout 5

fired=$(timers "$now" "$(date +%s)" LastTriggerUSec)
fired=${fired%, }
{
  cat <<EOF
# Benchmark results

The baseline of $(date +%Y-%m-%d) on an $(lscpu | sed -n 's/^Model name: *//p')
($(nproc) cores, $(free -g | awk '/^Mem:/ { print $2 }') GB), release build, from
\`bench/baseline.sh\`, which regenerates this file and the CSV files beside
it (\`bench/results/\`, one row per run); it took $((SECONDS / 3600)) h $((SECONDS % 3600 / 60)) min,
started at a load average of $load, and the scheduled jobs that ran
meanwhile were: ${fired:-none}.
The journal of \`linlog-baseline\` says every ten minutes what else used
a CPU. The sequential runs went in four
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
echo "finished $(date '+%Y-%m-%d %H:%M'): bench/RESULTS.md"
