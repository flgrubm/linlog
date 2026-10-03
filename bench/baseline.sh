#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Takes the benchmark baseline: CSV files in bench/results/DAY/, DAY being
# the day the baseline started, so that every baseline keeps a directory
# of its own; their tables in RESULTS.md beside them, whose header names
# the commit measured; and a copy of those tables in bench/RESULTS.md, the
# latest baseline's. Every family sequentially with its default engine and
# with every engine that applies, the net engine's test period, the
# intuitionistic mode of the families that have one, the whole LLTP
# library in both modes, and the intuitionistic library again under each
# atom bias alone; the largest sizes with 20 minutes each, the LLTP
# problems that ended at the copy bound or the recursion limit with those
# raised; then the thread counts 2, 4, 8 and every core on the hard
# families, and every core on the LLTP
# problems not decided at once; last, the runs of those that more room lets
# finish. The LLTP problems of the later stages are those of the first
# baseline (`reference'), so that every row of it has its counterpart.
# About ten hours on sixteen cores, on an otherwise idle machine (stages 1
# to 3 of the first baseline took 9 h 21 min on 2026-09-30).
#
#   nix build .#lltp -o bench/lltp          # once: the LLTP library
#   bench/baseline.sh --arm --fresh          # from the devshell
#   systemctl --user list-timers             # linlog-baseline and its stop
#   journalctl --user -fu linlog-baseline    # progress every ten minutes
#
# --arm takes the baseline unattended in the night the machine is the
# benchmark's (`slot', 20:00 to 07:00; --slot=HH:MM-HH:MM gives another):
# a transient user timer starts the unit below at the slot's start, or at
# once when the slot has begun, and another stops it at the slot's end
# whatever its state. The unit waits until the machine is idle (on mains
# and a load average of at most 1) and, from the last moment at which the
# estimated duration still ends within the slot, starts whatever the load,
# recording that it did. --detach starts the unit at once and never stops
# it.
#
# The unit, `linlog-baseline', survives the terminal: 40 GiB of memory and
# no swap for all of it, the kernel's OOM killer taking the runaway
# process alone (`OOMPolicy=continue', its run a crash row) rather than
# systemd stopping the unit, no core dumps, and its own target directory
# (target/baseline) so that builds in the checkout do not touch the binary
# it runs. It keeps the user's other slices off the performance cores and
# stops the user timers due during the run, and its ExecStopPost undoes
# both however the run ends. `systemctl --user stop linlog-baseline' stops
# it.
#
# Without --fresh the script resumes the latest baseline directory that
# has no RESULTS.md yet: every run appends to its CSV file and skips what
# the file already has (`--append --resume'), so running the script again
# after an interruption (the stop at the slot's end) finishes the baseline
# on another night. --into=DIR resumes DIR, finished or not, which adds a
# later version's new runs to an earlier baseline. --fresh deletes the
# directory of the day (or DIR) and starts it anew; other baselines are
# never touched. The script refuses to start on
# battery, on a busy machine (a load average above 1) or with a scheduled
# job due within the estimate (`systemctl list-timers': nix-gc at
# midnight, nix-optimise before four, backups) unless given --force, and
# --arm names those due in the slot: the timings are only worth what the
# machine's idleness is, and RESULTS.md records the load the run started
# with and the jobs that ran meanwhile. Stop such timers for the night
# (`sudo systemctl stop nix-gc.timer', start them again afterwards).
#
# The sequential runs go in four streams at once, each pinned to a core of
# its own with taskset; `cores' names four performance cores of the machine
# the baseline was taken on (an Intel Core Ultra X9 388H: CPUs 0 to 3 are
# its performance cores), so adjust it for another. The parallel runs come
# after them, with the machine to themselves, so that a run stopped at the
# slot's end has the sequential stages, which a performance pass compares
# with first, complete.
set -euo pipefail
self=$(realpath "$0")
cd "$(dirname "$self")/.."
lltp=bench/lltp
results=bench/results
cores=(1 0 2 3)
slot=20:00-07:00
estimate=$((10 * 3600))
# The first baseline, whose intuitionistic LLTP pass chooses the problems
# that the later stages run again: chosen from each baseline's own pass,
# the sets would differ between baselines and no row would compare.
reference=$results/2026-09-30

# The user slices a detached run keeps off the performance cores, and the
# file listing the user timers it stopped for its duration.
slices=(app.slice session.slice background.slice)
paused=${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/linlog-baseline-timers
unshield() {
  local slice
  for slice in "${slices[@]}"; do
    systemctl --user set-property --runtime "$slice" AllowedCPUs= || true
  done
  if [ -e "$paused" ]; then
    xargs -r systemctl --user start <"$paused" || true
    rm -f "$paused"
  fi
}
# The unit's ExecStopPost: the slices get every core back and the user
# timers start again however the run ended, since a stopped unit's
# processes may be killed before a trap of theirs has run.
if [ "${1:-}" = --unshield ]; then
  unshield
  exit
fi

detach=false arm=false fresh=false force=false start_by='' into=''
for arg; do
  case $arg in
  --detach) detach=true ;;
  --arm) arm=true ;;
  --slot=*) slot=${arg#--slot=} ;;
  --fresh) fresh=true ;;
  --force) force=true ;;
  # The unit's: wait for an idle machine until then (seconds since the
  # epoch).
  --start-by=*) start_by=${arg#--start-by=} ;;
  --into=*) into=${arg#--into=} ;;
  *)
    echo "usage: bench/baseline.sh [--arm [--slot=HH:MM-HH:MM] | --detach] [--fresh] [--into=DIR] [--force]" >&2
    exit 2
    ;;
  esac
done

# The baseline's directory: the one --into names (a finished baseline is
# resumed too, its new runs added), else the latest one without
# RESULTS.md, a baseline that has not finished, or else today's.
out=${into:-$results/$(date +%F)}
if [ -z "$into" ] && ! $fresh; then
  for dir in "$results"/*/; do
    if [ -d "$dir" ] && [ ! -e "$dir/RESULTS.md" ]; then
      out=${dir%/}
    fi
  done
fi

# The processes other than the benchmark's that use a CPU now, by the
# second of two samples of top.
others() {
  top -b -n 2 -d 2 -o %CPU -w 200 | awk '
    /^top -/ { frame++ }
    frame == 2 && $1 ~ /^[0-9]+$/ && $9 + 0 >= 1 && $12 !~ /^(linlog-bench|top)$/ {
      printf "%s%s %s%%", sep, $12, $9
      sep = ", "
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
        case $unit in logrotate.timer | fwupd-refresh.timer | systemd-tmpfiles-clean.timer | linlog-baseline*) continue ;; esac
        t=$(systemctl "$scope" show "$unit" -p "$3" --value --timestamp=unix)
        t=${t#@}
        if [ -n "$t" ] && [ "$t" -ge "$1" ] && [ "$t" -le "$2" ]; then
          printf '%s %s, ' "${unit%.timer}" "$(date -d "@$t" +%H:%M)"
        fi
      done
  done
}

# Whether the machine runs on mains power, or has no battery to run on: on
# battery its idle manager suspends it after twenty minutes, and the
# clocks follow the battery's profile.
on_mains() {
  local supply
  for supply in /sys/class/power_supply/*; do
    if [ "$(cat "$supply/type")" = Mains ] && [ "$(cat "$supply/online")" = 1 ]; then
      return 0
    fi
  done
  ! ls /sys/class/power_supply/*/capacity >/dev/null 2>&1
}

# The package's thermal throttling so far, as "EVENTS MILLISECONDS"; the
# counters of CPU 0 are the whole package's.
throttling() {
  local dir=/sys/devices/system/cpu/cpu0/thermal_throttle
  if [ -r "$dir/package_throttle_count" ]; then
    echo "$(cat "$dir/package_throttle_count") $(cat "$dir/package_throttle_total_time_ms")"
  else
    echo "0 0"
  fi
}

# The frequency settings, in words.
settings() {
  local cpu=/sys/devices/system/cpu/cpu0/cpufreq turbo=on
  if [ "$(cat /sys/devices/system/cpu/intel_pstate/no_turbo 2>/dev/null)" = 1 ]; then
    turbo=off
  fi
  echo "platform profile $(cat /sys/firmware/acpi/platform_profile 2>/dev/null), governor $(cat $cpu/scaling_governor 2>/dev/null), energy preference $(cat $cpu/energy_performance_preference 2>/dev/null), turbo $turbo"
}

# Whether the load average of the last minute is above 1.
busy() {
  awk -v l="$(cut -d' ' -f1 /proc/loadavg)" 'BEGIN { exit !(l > 1) }'
}

# The current or next slot as "START END", in seconds since the epoch;
# START is now once the slot has begun.
slot_times() {
  local from=${slot%-*} to=${slot#*-} now day start length
  now=$(date +%s)
  length=$((($(date -d "$to" +%s) - $(date -d "$from" +%s) + 86400) % 86400))
  for day in yesterday today tomorrow; do
    start=$(date -d "$day $from" +%s)
    if [ $((start + length)) -gt "$now" ]; then
      echo $((start > now ? start : now)) $((start + length))
      return
    fi
  done
}

# The commit the binary is built from, and the files the working copy
# changes beyond the baseline's own; read without a snapshot, which would
# sign a commit from inside the unit.
commit() {
  local changes
  changes=$(jj --ignore-working-copy diff --name-only -r @ |
    grep -v -E '^bench/(results/|RESULTS\.md)' | paste -sd ' ' || true)
  jj --ignore-working-copy log --no-graph -r @- \
    -T 'commit_id.short(12) ++ " (" ++ description.first_line() ++ ")"'
  if [ -n "$changes" ]; then
    printf ', with uncommitted changes to %s' "${changes:0:300}"
  fi
}

if $arm || $detach; then
  if systemctl --user is-active --quiet linlog-baseline.service; then
    echo "linlog-baseline runs already; stop it with: systemctl --user stop linlog-baseline" >&2
    exit 1
  fi
  systemctl --user stop linlog-baseline.timer linlog-baseline-stop.timer 2>/dev/null || true
  systemctl --user reset-failed linlog-baseline.service linlog-baseline.timer \
    linlog-baseline-stop.service linlog-baseline-stop.timer 2>/dev/null || true
  if $fresh; then
    rm -rf "$out"
  fi
  # The directory exists, so that the unit's run resumes it; the snapshot
  # is the working copy the unit reads its commit from.
  mkdir -p "$out"
  jj st >/dev/null
  # A slice of its own, so that the other user slices can be kept off the
  # performance cores; an inhibitor, so that neither the idle manager nor a
  # closed lid suspends the machine.
  unit=(systemd-run --user --unit=linlog-baseline --slice=linlog.slice --same-dir -p MemoryMax=40G
    -p MemorySwapMax=0 -p OOMPolicy=continue -p LimitCORE=0 --setenv=PATH="$PATH"
    -p ExecStopPost="$self --unshield"
    --setenv=CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target/baseline}")
  script=(systemd-inhibit --what=sleep:idle:handle-lid-switch --who=linlog-baseline
    --why="the benchmark baseline" "$self" --force --into="$out")
  if $detach; then
    "${unit[@]}" "${script[@]}"
    echo "into $out; follow it with: journalctl --user -fu linlog-baseline"
    exit
  fi
  read -r start end < <(slot_times)
  latest=$((end - estimate > start ? end - estimate : start))
  when=(--on-calendar="$(date -d "@$start" '+%F %T')")
  if [ "$start" -le $(($(date +%s) + 5)) ]; then
    when=(--on-active=5)
  fi
  due=$(timers "$start" "$end" NextElapseUSecRealtime)
  "${unit[@]}" "${when[@]}" --timer-property=AccuracySec=1s "${script[@]}" --start-by="$latest"
  systemd-run --user --unit=linlog-baseline-stop --on-calendar="$(date -d "@$end" '+%F %T')" \
    --timer-property=AccuracySec=1s "$(command -v systemctl)" --user stop \
    linlog-baseline.timer linlog-baseline.service
  # The estimate is a whole baseline's; a resumed one runs only what its
  # CSV files lack, which may be minutes.
  length="takes about $((estimate / 3600)) h $((estimate % 3600 / 60)) min"
  if compgen -G "$out/*.csv" >/dev/null; then
    length="runs only what the directory's CSV files lack (a whole baseline ${length})"
  fi
  echo "armed, into $out: it starts at $(date -d "@$start" '+%a %H:%M') once the machine is idle, at $(date -d "@$latest" +%H:%M) whatever the load, $length and is stopped at $(date -d "@$end" '+%a %H:%M')"
  if [ -n "$due" ]; then
    echo "timers due in the slot: ${due%, }; stop them for the night"
  fi
  on_mains || echo "the machine runs on battery: plug it in before the slot"
  exit
fi

# Armed, the run waits for an idle machine on mains until the latest
# start, and then starts regardless.
forced=
if [ -n "$start_by" ]; then
  waited=false
  while ! on_mains || busy; do
    if [ "$(date +%s)" -ge "$start_by" ]; then
      forced=", not idle at the latest start ($(on_mains && echo mains || echo BATTERY); $(others))"
      break
    fi
    $waited || echo "$(date +%H:%M) waiting for an idle machine on mains until $(date -d "@$start_by" +%H:%M): load $(cut -d' ' -f1-3 /proc/loadavg); $(others)"
    waited=true
    sleep 60
  done
  SECONDS=0
fi

read -r load _ </proc/loadavg
if ! $force && ! on_mains; then
  echo "the machine runs on battery: plug it in, or pass --force" >&2
  exit 1
fi
if ! $force && busy; then
  echo "the machine is busy (load average $load; $(others)); start when it is idle, or pass --force" >&2
  exit 1
fi
# A scheduled job during the run (nix-gc at midnight, nix-optimise before
# four, a backup) competes with it for a minute or so of CPU and disk.
now=$(date +%s)
due=$(timers "$now" $((now + estimate)) NextElapseUSecRealtime)
due=${due%, }
if ! $force && [ -n "$due" ]; then
  echo "timers fire during the run: $due; start after them, stop them for the night, or pass --force" >&2
  exit 1
fi
if $fresh; then
  rm -rf "$out"
fi
mkdir -p "$out"

# Every start of the run, a line in starts.txt: a resumed baseline has
# several, which RESULTS.md lists.
record="$(date '+%Y-%m-%d %H:%M'), commit $(commit), load average $load${forced}"
echo "$record" >>"$out/starts.txt"
echo "started into $out: $record; timers due: ${due:-none}; $(settings)"
read -r throttled_before throttled_ms_before < <(throttling)

# In its own slice, the run keeps the user's other slices (the desktop,
# the editor, the sync clients) off the performance cores its sequential
# streams are pinned to, and lets them back when it ends; the system's
# services and kernel threads it cannot move (.claude/rules/bench.md says
# how an administrator can).
if grep -q linlog.slice /proc/self/cgroup; then
  for slice in "${slices[@]}"; do
    systemctl --user set-property --runtime "$slice" AllowedCPUs=4-15
  done
  echo "kept ${slices[*]} off CPUs ${cores[*]}"
  # The user's own timers due during the run wait for its end.
  systemctl --user list-timers --no-legend | awk '{ print $(NF - 1) }' |
    while read -r unit; do
      case $unit in linlog-baseline* | systemd-tmpfiles-clean.timer) continue ;; esac
      t=$(systemctl --user show "$unit" -p NextElapseUSecRealtime --value --timestamp=unix)
      t=${t#@}
      if [ -n "$t" ] && [ "$t" -le $((now + estimate)) ]; then
        systemctl --user stop "$unit"
        echo "$unit" >>"$paused"
        echo "stopped $unit until the run ends"
      fi
    done
fi

cargo build --release --locked --package linlog-bench
bench=${CARGO_TARGET_DIR:-target}/release/linlog-bench

# Every ten minutes: the last progress line of every run (with its estimate
# of the time left), the load and whatever else uses a CPU.
progress() {
  while sleep 600; do
    local events ms power=mains
    read -r events ms < <(throttling)
    on_mains || power=BATTERY
    echo "== $(date +%H:%M), $((SECONDS / 60)) min in, load $(cut -d' ' -f1-3 /proc/loadavg), $power, throttled $((events - throttled_before)) times for $(((ms - throttled_ms_before) / 1000)) s so far; others: $(others)"
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
# `-`. Every process gets `cap' GiB of address space, 12 until stage 4, so
# that a run whose memory grows without bound (the check of a large proof
# takes gigabytes) fails alone; three such processes at once stay within
# the unit's 40 GiB.
cap=12
run() {
  local cpu=$1 name=$2
  shift 2
  local pin=(prlimit --as=$((cap << 30)))
  if [ "$cpu" != - ]; then
    pin+=(taskset -c "$cpu")
  fi
  echo "$(date +%H:%M) $name"
  "${pin[@]}" "$bench" run "$@" --output "$out/$name.csv" --append --resume 2>>"$out/$name.log"
}

repeat=(--repeat 3 --repeat-under 2)
mll=(--family "partition-yes=4,5,6,12,20" --family "partition-no=3,4,9,12" --family 3-partition-mll-yes
  --family 3-partition-mll-no --family wide-m1 --family wide-m2 --family "wide-m3=12,24,30,256,2048"
  --family "wide-m4=12,24,28,256,2048")

# Stage 1, sequential, four streams: every family, the engines against each
# other, the net engine's test period, the intuitionistic mode, the whole
# LLTP library in both modes, and the intuitionistic library under each
# atom bias alone: the backward search (the rarer literal positive) and the
# forward one (its factors positive) within the forward bound of the
# default, which runs the two on a sequent with exponentials. The two
# passes under one bias are the longest and have a stream each; the others
# share theirs, the families after one library pass and the engines after
# the other. About three and a half hours.
streams=()
run "${cores[0]}" lltp-rarer --lltp "$lltp/ILL" --bias rarer --timeout 5 &
streams+=($!)
run "${cores[1]}" lltp-forward --lltp "$lltp/ILL" --bias factors --copies 30 --timeout 5 &
streams+=($!)
(
  run "${cores[2]}" lltp-intuitionistic --lltp "$lltp/ILL" --timeout 5
  run "${cores[2]}" families --all-families --timeout 300 "${repeat[@]}"
) &
streams+=($!)
(
  # In reverse, so that this pass loads the library's largest files (up to
  # 103 MB each) at other times than the three in order.
  run "${cores[3]}" lltp-classical --lltp "$lltp/CLL" --lltp "$lltp/ILL" --modes classical \
    --reverse --timeout 5
  run "${cores[3]}" engines "${mll[@]}" --family additive --problems bench/problems/slow-tests.txt \
    --engines focus,net --timeout 60 "${repeat[@]}"
  for period in 1 2 8 16; do
    run "${cores[3]}" period-$period --family wide-m1=256,2048 --family wide-m2=256,2048 \
      --family 3-partition-mll-no=4,5 --family partition-yes=4 --family partition-no=3,4 \
      --engines net --test-period $period --timeout 60 "${repeat[@]}"
  done
  run "${cores[3]}" intuitionistic --family counter --family counter-over --family chain \
    --family 3-partition-yes --family 3-partition-no=4 --modes intuitionistic --timeout 300 \
    "${repeat[@]}"
) &
streams+=($!)
wait "${streams[@]}"

# The intuitionistic LLTP problems whose pass in the first baseline ended
# with the reason given (or, for `slow', timed out, was killed or took
# 50 ms or more), as a list for --only: the problems a knob can change.
ended() {
  awk -F, -v reason="$1" 'NR > 1 && ($16 == reason || (reason == "slow" &&
    ($16 ~ /^(timeout|killed)$/ || ($15 ~ /proved|unprovable/ && $22 >= 50)))) { print $5 }' \
    "$reference/lltp-intuitionistic.csv" | sort -u | paste -sd,
}

# Stage 2, sequential, four streams: the largest instances that time out at
# 300 s, once each with 20 minutes, for the times a performance pass has
# to beat (the first baseline's, decided in milliseconds since, and the
# sizes added after it); and the LLTP problems that ended at the copy bound
# or at the recursion limit, again with a bound of 10 and a limit of 16384.
# About an hour.
streams=()
run "${cores[0]}" long-1 --family 3-partition-no=5 --family partition-no=5,15 \
  --family partition-yes=7,28 --family counter=16,64 --timeout 1200 &
streams+=($!)
run "${cores[1]}" long-2 --family mix=11 --family wide-m3=36 --family wide-m4=36 \
  --family qbf=24,48 --only "qbf/24#0,qbf/48#0,mix,wide" --timeout 1200 &
streams+=($!)
run "${cores[2]}" lltp-copies-10 --lltp "$lltp/ILL" --only "$(ended copy_bound)" --copies 10 \
  --timeout 5 &
streams+=($!)
run "${cores[3]}" lltp-recursion --lltp "$lltp/ILL" --only "$(ended recursion_limit)" \
  --recursion-limit 16384 --timeout 5 &
streams+=($!)
wait "${streams[@]}"

# Stage 3, alone on the machine: the hard families on 1 (the speedups'
# baseline, taken alone like the rest), 2, 4, 8 and every core, the net
# engine on the same thread counts, and the LLTP problems that are not decided at once on
# every core (the command's default). About three and a half hours.
run - parallel --family 3-partition-yes --family 3-partition-no --family partition-yes=5,6,7,20,24 \
  --family partition-no=4,5,12,14 --family qbf=16,20,24,40,44 --family mix=8,9,10,11 --family counter \
  --family counter-over --family wide-m3=24,30,36 --family wide-m4=28,32,36 \
  --jobs 1,2,4,8,all --timeout 120 "${repeat[@]}"
# Two runs, since --only would drop the family.
run - parallel-net --family 3-partition-mll-no=4,5,6 --engines net --jobs 1,2,4,8,all --timeout 60 \
  "${repeat[@]}"
run - parallel-net --problems bench/problems/slow-tests.txt --only partition-table --engines net \
  --jobs 1,2,4,8,all --timeout 60 "${repeat[@]}"
slow=$(ended slow)
run - lltp-all-cores --lltp "$lltp/ILL" --only "$slow" --jobs all --timeout 5

# Stage 4: the runs above that were killed or crashed and that more room
# lets finish, as bench/reruns.txt lists them (lines `FILE FAMILY/NAME`:
# the CSV file of the run, the problem), again into FILE-generous.csv. On
# one thread, in three streams, ten minutes' grace after the time limit and
# 16 GiB a process: the library's largest files take up to 16 s to load,
# and a search that misses its stop inside a long split enumeration stops
# when the enumeration ends. On every core, alone, a minute's grace and 32
# GiB: loading again, and a pool's memory. Minutes where every search
# stops at its limit, up to three hours where none does.
reruns() {
  awk -v file="$1" '$1 == file { print $2 }' bench/reruns.txt | paste -sd,
}
# again CPU FILE GRACE ARGS...: the reruns of FILE, if it has any.
again() {
  local cpu=$1 file=$2 grace=$3 only
  shift 3
  only=$(reruns "$file")
  if [ -n "$only" ]; then
    run "$cpu" "$file-generous" --only "$only" --grace "$grace" "$@"
  fi
}
cap=16
streams=()
again "${cores[0]}" lltp-intuitionistic 600 --lltp "$lltp/ILL" --timeout 5 &
streams+=($!)
again "${cores[1]}" lltp-classical 600 --lltp "$lltp/CLL" --lltp "$lltp/ILL" --modes classical \
  --timeout 5 &
streams+=($!)
again "${cores[2]}" lltp-recursion 600 --lltp "$lltp/ILL" --recursion-limit 16384 --timeout 5 &
streams+=($!)
wait "${streams[@]}"
cap=32
again - lltp-all-cores 60 --lltp "$lltp/ILL" --jobs all --timeout 5

fired=$(timers "$now" "$(date +%s)" LastTriggerUSec)
fired=${fired%, }
read -r throttled_after throttled_ms_after < <(throttling)
{
  cat <<EOF2
# Benchmark results

The baseline of ${out##*/} on an $(lscpu | sed -n 's/^Model name: *//p')
($(nproc) cores, $(free -g | awk '/^Mem:/ { print $2 }') GB), release build, from
\`bench/baseline.sh\`, which writes this file, the CSV files beside it
(\`$out/\`, one row per run) and a copy of this file as
\`bench/RESULTS.md\`, the latest baseline's. It started (once more for
every resumption), with the commit it measured:

$(sed 's/^/- /' "$out/starts.txt")

Its last part took $((SECONDS / 3600)) h $((SECONDS % 3600 / 60)) min, and the scheduled jobs that ran
meanwhile were: ${fired:-none}. The package throttled
$((throttled_after - throttled_before)) times for $(((throttled_ms_after - throttled_ms_before) / 1000)) s in that part
($(settings)).
The journal of \`linlog-baseline\` says every ten minutes what else used
a CPU. The sequential runs went in four
streams at once, each pinned to a performance core of its own; the
parallel runs had the machine to themselves. A time is the median of up to
three runs; \`✓\` is proved, \`✗\` refuted, \`?\` unknown (\`bound\`: the copy
bound, \`wide\`: a context too wide to split, \`depth\`: the recursion
limit), \`>\` a time limit reached, \`MISMATCH\` a verdict against the
problem's known one (for LLTP, the one its header claims). \`linlog-bench
summary $out/*.csv\` prints the tables again.

EOF2
  "$bench" summary "$out"/*.csv
} >"$out/RESULTS.md"
cp "$out/RESULTS.md" bench/RESULTS.md
echo "finished $(date '+%Y-%m-%d %H:%M'): $out/RESULTS.md and bench/RESULTS.md"
