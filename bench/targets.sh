#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Runs the target set of the focused engine's performance work: the
# instances the first baseline showed it losing on, sequentially, into
# bench/targets/LABEL.csv, so that
#
#   bench/targets.sh before        # at the commit to compare with
#   bench/targets.sh after         # at the commit to judge
#   linlog-bench summary bench/targets/before.csv bench/targets/after.csv
#
# prints the two side by side. On one thread the engine's counters
# (`nodes', `splits', `memo_hits', `memo_entries') are a function of the
# input, so they compare whatever the machine's load; `cpu_ms' compares as
# far as `wait_ms' stays small.
#
# The set is small enough for a shared machine by day: two streams, each
# pinned to a performance core of its own (`cores'), no run over 300 s,
# about three quarters of an hour at the commit the first baseline
# measured. It runs as the user unit `linlog-targets', which survives the
# terminal, with 8 GiB and no swap for all of it and 6 GiB of address
# space per process, so that a search whose memory grows without bound
# fails its own run. `journalctl --user -fu linlog-targets' follows it and
# `systemctl --user stop linlog-targets' stops it; the unit is gone when
# the file bench/targets/LABEL.csv exists. A run of a label that has rows
# already (in LABEL.1.csv and LABEL.2.csv, the streams' files) resumes it.
#
# The LLTP problems need the library: nix build .#lltp -o bench/lltp
set -euo pipefail
self=$(realpath "$0")
cd "$(dirname "$self")/.."
label=${1:?usage: bench/targets.sh LABEL}
lltp=bench/lltp
out=bench/targets
cores=(2 3)

if [ ! -d "$lltp/ILL" ]; then
  echo "no LLTP library at $lltp: nix build .#lltp -o bench/lltp" >&2
  exit 1
fi

if [ "${2:-}" != --inside ]; then
  # A copy of the binary, so that builds in the checkout meanwhile do not
  # touch the one the unit runs.
  cargo build --release --locked --package linlog-bench
  install -D "${CARGO_TARGET_DIR:-target}/release/linlog-bench" "target/targets/$label/linlog-bench"
  mkdir -p "$out"
  systemctl --user reset-failed linlog-targets.service 2>/dev/null || true
  systemd-run --user --unit=linlog-targets --same-dir --collect \
    -p MemoryMax=8G -p MemorySwapMax=0 -p OOMPolicy=continue -p LimitCORE=0 \
    --setenv=PATH="$PATH" "$self" "$label" --inside
  echo "into $out/$label.csv; follow it with: journalctl --user -fu linlog-targets"
  exit
fi

bench=target/targets/$label/linlog-bench

# run STREAM ARGS...: one `run' into the stream's file, on the stream's
# core.
run() {
  local stream=$1
  shift
  prlimit --as=$((6 << 30)) taskset -c "${cores[stream - 1]}" "$bench" run "$@" \
    --output "$out/$label.$stream.csv" --append --resume 2>>"$out/$label.log"
}

# Runs under ten seconds are taken three times, for the median.
repeat=(--repeat 3 --repeat-under 10)

# The intuitionistic LLTP problems that ended, in the first baseline's pass
# at 5 s (bench/results/2026-09-30/lltp-intuitionistic.csv), at the time
# limit, at the copy bound, at the recursion limit and with a context too
# wide to split: of each, every thirty-fifth to forty-first of those with
# at most 200 000 occurrences, in the order of their names.
timeout=(
  AirplaneLD-pt-0010_20_1 BridgeAndVehicles-V10-P10-N10-unfolded_1_1
  ClientsAndServers-0001-0_20_1 CloudDeployment_deploy_4_a_5_1 Dekker_dekker-10_5_1
  DES_des_30_a_10_1 DLCround_dlcro_03_b_50_1 DNAwalker_dnawalk-10_1_1
  Eratosthenes_eratosthenes-050_5_1 FMS-10_10_1 HexagonalGrid_hxg_126_5_1
  IOTPpurchase_IOTP_c1m1p1d1_20_1 LamportFastMutEx_lamport_fmea-4_10_1
  NeighborGrid_z_4d_3n_2m_c_2_3_20_1 ParamProductionCell_open_system_0_20_1 Peterson-4_50_1
  PolyORBLF_PolyORB-LF-S02-J04-T06-unfolded_10_1
  QuasiCertifProtocol_QCertifProtocol_10-unfold_20_1 Railroad_railroad-010-pt_20_1
  ResAllocation_RAS-R-10_10_1 SafeBus-03-unfolded_20_1 SmallOperatingSystem-MT0032DC0008_1_1
  TCPcondis_tcp15_20_1 TokenRing-10-unfolded_100_1
)
copy_bound=(
  ILLTP-SYJ-01/SYJ201+1.007 ILLTP-SYJ-01/SYJ203+1.016 ILLTP-SYJ-01/SYJ205+1.013
  ILLTP-SYJ-01/SYJ207+1.018 ILLTP-SYJ-01/SYJ210+1.006 ILLTP-SYJ-01/SYJ212+1.002
  ILLTP-SYJ-cbn/SYJ201+1.019 ILLTP-SYJ-cbn/SYJ204+1.010 ILLTP-SYJ-cbn/SYJ206+1.009
  ILLTP-SYJ-cbn/SYJ209+1.004 ILLTP-SYJ-cbn/SYJ210+1.020 ILLTP-SYJ-cbv/SYJ201+1.019
  ILLTP-SYJ-cbv/SYJ204+1.017 ILLTP-SYJ-cbv/SYJ207+1.004 ILLTP-SYJ-cbv/SYJ209+1.014
  ILLTP-SYJ-cbv/SYJ212+1.010 KLE-01/KLE058+1 KLE-cbn/KLE084+1 KLE-IMP-CONJ/KLE_26_01
  KLE-IMP-CONJ/KLE_57_CBV petri-nets/MCC/AutoFlight_afcs_03_b_5_1
  petri-nets/MCC/DES_des_01_b_10_1 petri-nets/MCC/DES_des_50_b_20_1 petri-nets/MCC/IBM319_5_1
)
recursion_limit=(
  ILLTP-SYJ-01/SYJ208+1.015 ILLTP-SYJ-cbv/SYJ208+1.016 petri-nets/MCC/AutoFlight_afcs_96_b_1_1
  petri-nets/MCC/BridgeAndVehicles-V50-P20-N20-unfolded_5_1
  petri-nets/MCC/BridgeAndVehicles-V80-P20-N50-unfolded_1_1
  petri-nets/MCC/CloudDeployment_deploy_5_b_100_1
  petri-nets/MCC/CloudReconfiguration_reconf_3_04_20_1
  petri-nets/MCC/CloudReconfiguration_reconf_3_10_100_1
  petri-nets/MCC/CloudReconfiguration_reconf_3_15_20_1
  petri-nets/MCC/CloudReconfiguration_reconf_4_01_100_1
  petri-nets/MCC/Diffusion2D_2D8_gradient_20x20_100_5_1
  petri-nets/MCC/Diffusion2D_2D8_gradient_40x40_50_5_1 petri-nets/MCC/DLCround_dlcro_07_b_5_1
  petri-nets/MCC/DLCround_dlcro_12_a_1_1 petri-nets/MCC/DLCshifumi_dlcsh_4_a_100_1
  petri-nets/MCC/Echo_echo-d5r3_5_1 petri-nets/MCC/FlexibleBarrier_flexbar_18_b_5_1
  petri-nets/MCC/HypercubeGrid_hc3k4p4b12_5_1 petri-nets/MCC/NeoElection_neoelection-7.unf_10_1
  petri-nets/MCC/Philosophers-1000_1_1
  petri-nets/MCC/PolyORBLF_PolyORB-LF-S04-J04-T08-unfolded_5_1
  petri-nets/MCC/Railroad_railroad-050-pt_50_1 petri-nets/MCC/RwMutex_rwmutex-r2000w10_1_1
  petri-nets/MCC/SmallOperatingSystem-MT2048DC1024_1_1
)
too_wide=(
  Angiogenesis_angiogenesis-15_20_1 AutoFlight_afcs_48_a_50_1
  CircadianClock_circadian_clock-001000_1_1 ClientsAndServers-0010-1_100_1
  ClientsAndServers-0020-3_50_1 Diffusion2D_2D8_gradient_5x5_150_20_1 Echo_echo-d2r9_100_1
  ERK_erk-001000_1_1 GPPP_G-PPP-100-1000_100_1 GPPP_G-PPP-1-1000_50_1
  HouseConstruction-100_5_1 JoinFreeModules_joinFree-10_50_1 Kanban-500_50_1 MAPK-80_20_1
  PermAdmissibility_unf-8x8-4stageSEN-50_10_1 Planning_planning_5_1
  QuasiCertifProtocol_QCertifProtocol_32-unfold_20_1 ResAllocation_RAS-C-50_100_1
  RobotManipulation_robot-manipulation-500_10_1 SmallOperatingSystem-MT0064DC0016_1_1
  SmallOperatingSystem-MT0512DC0128_1_1 Solitaire_soli2_counter_20_1 SwimmingPool-4_10_1
  SwimmingPool-9_5_1
)
# Petri nets whose search, in the first baseline, missed its stop inside a
# split enumeration and ran 10 s to over 400 s under a limit of 5 s
# (bench/reruns.txt and the `-generous' rows): here with 25 s before the
# kill, which a run that still misses its stop meets.
missed_stop=(
  AutoFlight_afcs_05_a_1_1 ClientsAndServers-0005-0_1_1 IOTPpurchase_IOTP_c5m4p3d2_1_1
  RwMutex_rwmutex-r10w100_1_1 PermAdmissibility_unf-8x8-4stageSEN-05_100_1
  PhaseVariation_5-10_phaseVariation_50_1 SimpleLoadBal_simple_lbs-15_50_1 DES_des_00_a_20_1
  SafeBus-10-unfolded_1_1 TCPcondis_tcp30_50_1 RobotManipulation_robot-manipulation-5_1_1
  Parking_parking_2_8_5_1 ResAllocation_RAS-R-15_10_1
)
# Under a recursion limit of 16 384: two nets proved long after their
# limit, one whose proof arena outgrew the memory inside a split
# enumeration and one killed after 605 s.
deep=(
  TokenRing-15-unfolded_1_1 TokenRing-20-unfolded_1_1 Diffusion2D_2D8_gradient_20x20_50_1_1
  DatabaseWithMutex_database20UNFOLD_5_1
)

# names PREFIX NAME...: the problems as a list for --only.
names() {
  local prefix=$1 name list=
  shift
  for name in "$@"; do
    list+=",$prefix$name.p"
  done
  echo "${list#,}"
}

# The first stream: the Horn encodings, Mix, and the LLTP problems.
(
  run 1 --family 3-partition-no=4,5 --family partition-yes=5,6,7 --family partition-no=4,5 \
    --family mix=8,9,10,11 --timeout 300 "${repeat[@]}"
  run 1 --lltp "$lltp/ILL" --timeout 5 --only "$(names ILL/petri-nets/MCC/ "${timeout[@]}")"
  run 1 --lltp "$lltp/ILL" --timeout 5 --only "$(names ILL/ "${copy_bound[@]}")"
  run 1 --lltp "$lltp/ILL" --timeout 5 --only "$(names ILL/ "${recursion_limit[@]}")"
  run 1 --lltp "$lltp/ILL" --timeout 5 --only "$(names ILL/petri-nets/MCC/ "${too_wide[@]}")"
  run 1 --lltp "$lltp/ILL" --timeout 5 --grace 25 \
    --only "$(names ILL/petri-nets/MCC/ "${missed_stop[@]}")"
  run 1 --lltp "$lltp/ILL" --timeout 5 --grace 55 --recursion-limit 16384 \
    --only "$(names ILL/petri-nets/MCC/ "${deep[@]}")"
) &
first=$!

# The second: QBF, the Petri-net counters in both modes, the exponential
# families, the wide sequents (which the dispatch sends to the focused
# engine), and the problem file.
(
  run 2 --family qbf=16,20 --timeout 300 "${repeat[@]}"
  run 2 --family counter=8,16 --family counter-over=8 --modes classical,intuitionistic \
    --timeout 300 "${repeat[@]}"
  run 2 --family growing --family chain --family wide-m3=24,30 --family wide-m4=24,28 \
    --timeout 300 "${repeat[@]}"
  run 2 --problems bench/problems/slow-tests.txt --timeout 300 "${repeat[@]}"
) &
second=$!
wait "$first" "$second"

{
  head -n 1 "$out/$label.1.csv"
  tail -q -n +2 "$out/$label.1.csv" "$out/$label.2.csv"
} >"$out/$label.csv"
rm "$out/$label.1.csv" "$out/$label.2.csv"
