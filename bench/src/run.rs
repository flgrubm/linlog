// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Running problems. The parent (`run`) starts this program again as a
//! child (`one`) for every run, so that a crash, a stack overflow or a run
//! that ignores its time limit costs that run only; the child times the
//! search alone, not its start or the parsing, and prints the tail of the
//! run's CSV row: once with the search's verdict, before it checks a
//! proof, and once more with the check's result. A child that takes too
//! long to load its problem, or outlives its time limit by more than the
//! grace period counted from the end of the load, is killed; one that
//! dies in its check leaves the row its verdict.

use crate::problems::{self, Reference, mode_name};
use crate::{OneArgs, RunArgs};
use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;
use linlog::search::{Engine, Options, Reason, Verdict, prove_until};
use linlog::{Atom, Bias, Error, Forest, Mode, Sign};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

/// The columns the parent writes, then those the child prints. Files
/// from before the portfolio of worker orders was removed have a column
/// `portfolio` after `jobs`, which the summaries still read.
pub const HEADER: &str = "source,family,size,index,problem,mode,engine_requested,jobs,\
                          test_period,timeout_s,run,copies,expected,verdict,reason,checked,engine,\
                          fragment,occurrences,multiplicity,time_ms,nodes,memo_hits,memo_entries,\
                          splits,links,tests,recursion_limit,cpu_ms,wait_ms,bias,forward_copies,\
                          check_ms,memory_limit";

/// The columns the child prints.
const TAIL: usize = 23;

/// The line the child prints when its problem is loaded and its search
/// starts, from which the parent counts the time limit.
const LOADED: &str = "loaded";

/// The tail's column that says how the check of a proof went.
const CHECKED: usize = 4;

/// Which mode to run a problem in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ModeChoice {
    /// The problem's own
    Given,
    /// The problem's own made classical
    Classical,
    /// The problem's own made intuitionistic (without Mix)
    Intuitionistic,
}

impl ModeChoice {
    /// Returns the mode to run a problem meant for `given` in.
    fn apply(self, given: Mode) -> Mode {
        match self {
            Self::Given => given,
            Self::Classical => Mode {
                intuitionistic: false,
                ..given
            },
            Self::Intuitionistic => Mode {
                intuitionistic: true,
                mix: false,
                ..given
            },
        }
    }
}

/// How the focused engines pick the positive literal of every atom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum BiasChoice {
    /// By tensor factors without exponentials, the rarer literal with them
    Auto,
    /// The rarer literal
    Rarer,
    /// By tensor factors
    Factors,
}

impl BiasChoice {
    /// Returns the rule of the library.
    fn bias(self) -> Bias {
        match self {
            Self::Auto => Bias::Auto,
            Self::Rarer => Bias::Rarer,
            Self::Factors => Bias::Factors,
        }
    }
}

/// The memory bound of a run in bytes: the one asked for, none for 0, or
/// the library's default.
fn memory_limit(asked: Option<u64>) -> Option<u64> {
    match asked {
        None => Some(Options::DEFAULT_MEMORY_LIMIT),
        Some(0) => None,
        bound => bound,
    }
}

/// The memory bound of a run as its column has it: the bytes, or 0 for
/// none.
fn memory_column(asked: Option<u64>) -> String {
    memory_limit(asked).unwrap_or(0).to_string()
}

/// The forward search's copy bound of a run: the one asked for, or the
/// library's default.
fn forward_copies(asked: Option<u32>) -> u32 {
    asked.unwrap_or(Options::DEFAULT_FORWARD_COPIES)
}

/// Which engine to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum EngineChoice {
    /// The one the fragment and the mode call for
    Auto,
    /// The focused engine, one-sided
    Focus,
    /// The proof-net engine
    Net,
    /// The focused engine, two-sided
    TwoSided,
    /// The additive fast path
    Additive,
}

impl EngineChoice {
    /// Returns the engine to force, if any.
    fn engine(self) -> Option<Engine> {
        match self {
            Self::Auto => None,
            Self::Focus => Some(Engine::Focus),
            Self::Net => Some(Engine::Net),
            Self::TwoSided => Some(Engine::TwoSided),
            Self::Additive => Some(Engine::Additive),
        }
    }
}

/// The name of a value as its argument spells it.
fn name(value: &impl ValueEnum) -> String {
    value
        .to_possible_value()
        .map_or_else(String::new, |v| v.get_name().to_owned())
}

/// Runs every problem in every configuration asked for and writes the rows.
pub fn run(args: &RunArgs) -> Result<()> {
    let mut references = problems::families(&args.family, args.all_families)?;
    references.extend(problems::lltp(&args.lltp)?);
    references.extend(problems::files(&args.problems)?);
    if !args.only.is_empty() {
        references.retain(|r| {
            let path = format!("{}/{}", r.family, r.name);
            args.only.iter().any(|only| path.contains(only.as_str()))
        });
    }
    let jobs = args
        .jobs
        .iter()
        .map(|j| match j.as_str() {
            "all" => Ok(thread::available_parallelism().map_or(1, |n| n.get())),
            n => n.parse().with_context(|| format!("jobs `{n}`")),
        })
        .collect::<Result<Vec<usize>>>()?;
    // A search uses no more threads than the machine runs at once, so a
    // row with a larger count would name threads that never ran.
    if let Ok(machine) = thread::available_parallelism()
        && let Some(n) = jobs.iter().find(|&&n| n > machine.get())
    {
        anyhow::bail!(
            "--jobs {n}: this process may run {machine} threads at once, and a search uses no more"
        );
    }

    let done_before = match &args.output {
        Some(path) if args.resume => finished(path)?,
        _ => HashSet::new(),
    };
    let mut out: Box<dyn Write> = match &args.output {
        Some(path) => {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(args.append)
                .truncate(!args.append)
                .open(path)
                .with_context(|| format!("opening {}", path.display()))?;
            let empty = file.metadata()?.len() == 0;
            let mut file = Box::new(std::io::BufWriter::new(file));
            if empty {
                writeln!(file, "{HEADER}")?;
            }
            file
        }
        None => {
            println!("{HEADER}");
            Box::new(std::io::stdout())
        }
    };

    let exe = std::env::current_exe()?;
    if args.reverse {
        references.reverse();
    }
    let total = references.len() * args.modes.len() * args.engines.len() * jobs.len();
    let mut done = 0;
    // The configurations run by this invocation, for the estimate of the
    // time left: those a resumed run skips cost nothing.
    let (start, mut ran) = (Instant::now(), 0);
    for reference in &references {
        let mut modes: Vec<(ModeChoice, Mode)> = Vec::new();
        for &choice in &args.modes {
            let mode = choice.apply(reference.mode);
            if !modes.iter().any(|&(_, m)| m == mode) {
                modes.push((choice, mode));
            }
        }
        for &(choice, mode) in &modes {
            for &engine in &args.engines {
                for &threads in &jobs {
                    done += 1;
                    let key = [
                        reference.source,
                        &reference.family,
                        &reference.name,
                        mode_name(mode),
                        &name(&engine),
                        &threads.to_string(),
                        &args.test_period.map_or(String::new(), |p| p.to_string()),
                        &name(&args.bias),
                        &forward_copies(args.forward_copies).to_string(),
                        &memory_column(args.memory_limit),
                    ]
                    .join(",");
                    if done_before.contains(&key) {
                        continue;
                    }
                    ran += 1;
                    for run in 0..args.repeat {
                        let tail = child(&exe, reference, choice, engine, threads, args)?;
                        let fields: Vec<&str> = tail.split(',').collect();
                        let (verdict, time) = (fields[2], fields[9].parse().unwrap_or(0.0));
                        writeln!(
                            out,
                            "{},{},{},{},{},{},{},{threads},{},{},{run},{tail}",
                            reference.source,
                            reference.family,
                            reference.size.map_or(String::new(), |s| s.to_string()),
                            reference.index.map_or(String::new(), |i| i.to_string()),
                            reference.name,
                            mode_name(mode),
                            name(&engine),
                            args.test_period.map_or(String::new(), |p| p.to_string()),
                            args.timeout,
                        )?;
                        out.flush()?;
                        if run == 0 {
                            // The configurations left at this run's mean.
                            let left = start.elapsed().as_secs_f64() / f64::from(ran)
                                * (total - done) as f64
                                / 60.0;
                            eprintln!(
                                "[{done}/{total}] {} {} {} j{threads}: {verdict} {} {time:.3} ms, \
                                 about {left:.0} min left",
                                reference.name,
                                mode_name(mode),
                                name(&engine),
                                fields[3],
                            );
                        }
                        if time >= args.repeat_under * 1000.0 || verdict == "refused" {
                            break;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// The problems and configurations a CSV file has a row for, each as its
/// source, family, problem, mode, requested engine, jobs, test period,
/// bias and forward copy bound joined by commas; a file from before
/// the bias had a column ran them all with `auto`, and one from before the
/// forward bound had one ran the default bias as one search, which no
/// bound names, so every row of it is run again. A file with other
/// columns than [`HEADER`] is refused: the rows this run appends would
/// not fit it.
fn finished(path: &Path) -> Result<HashSet<String>> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(HashSet::new());
    };
    let mut lines = text.lines();
    let header = lines.next().unwrap_or("");
    if header != HEADER {
        anyhow::bail!(
            "{} has other columns than this version writes; resume into a new file",
            path.display()
        );
    }
    let columns: Vec<&str> = header.split(',').collect();
    let at = |column: &str| columns.iter().position(|c| *c == column);
    let key = [
        "source",
        "family",
        "problem",
        "mode",
        "engine_requested",
        "jobs",
        "test_period",
    ]
    .map(at);
    let Some(key) = key.into_iter().collect::<Option<Vec<usize>>>() else {
        anyhow::bail!("{} is not a CSV file of `run`", path.display());
    };
    let bias = at("bias");
    let forward = at("forward_copies");
    // A file from before the bound ran without one.
    let memory = at("memory_limit");
    Ok(lines
        .map(|line| {
            let fields: Vec<&str> = line.split(',').collect();
            let field = |i: usize| fields.get(i).copied().unwrap_or("");
            let bias = bias.map(field).filter(|b| !b.is_empty()).unwrap_or("auto");
            key.iter()
                .map(|&i| field(i))
                .chain([
                    bias,
                    forward.map_or("", field),
                    memory.map(field).filter(|m| !m.is_empty()).unwrap_or("0"),
                ])
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect())
}

/// Runs one problem in a child process and returns the tail of its row:
/// the child's own, or one that says how it died.
fn child(
    exe: &Path,
    reference: &Reference,
    mode: ModeChoice,
    engine: EngineChoice,
    jobs: usize,
    args: &RunArgs,
) -> Result<String> {
    let mut command = Command::new(exe);
    command
        .args(["one", "--problem", &reference.id])
        .args(["--mode", &name(&mode), "--engine", &name(&engine)])
        .args([
            "--jobs",
            &jobs.to_string(),
            "--timeout",
            &args.timeout.to_string(),
        ]);
    if let Some(copies) = args.copies {
        command.args(["--copies", &copies.to_string()]);
    }
    command.args(["--bias", &name(&args.bias)]);
    if let Some(copies) = args.forward_copies {
        command.args(["--forward-copies", &copies.to_string()]);
    }
    if let Some(bytes) = args.memory_limit {
        command.args(["--memory-limit", &bytes.to_string()]);
    }
    if let Some(limit) = args.recursion_limit {
        command.args(["--recursion-limit", &limit.to_string()]);
    }
    if let Some(period) = args.test_period {
        command.args(["--test-period", &period.to_string()]);
    }
    let mut process = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("starting a child")?;
    // The child's rows, and the moment it said its problem was loaded.
    let (loaded_at, loaded) = mpsc::channel();
    let pipe = process.stdout.take().expect("piped");
    let stdout = thread::spawn(move || {
        let mut rows = Vec::new();
        for line in BufReader::new(pipe).lines().map_while(Result::ok) {
            if line == LOADED {
                let _ = loaded_at.send(Instant::now());
            } else {
                rows.push(line);
            }
        }
        rows
    });
    let mut pipe = process.stderr.take().expect("piped");
    let stderr = thread::spawn(move || {
        let mut text = String::new();
        let _ = pipe.read_to_string(&mut text);
        text
    });

    let spawned = Instant::now();
    let mut searching = None;
    // Time for the pool's teardown and the proof check.
    let grace = args.grace.unwrap_or(args.timeout * 0.1 + 5.0);
    let limit = Duration::from_secs_f64(args.timeout + grace);
    let load_limit = Duration::from_secs_f64(args.load_limit);
    let status = loop {
        if searching.is_none() {
            searching = loaded.try_recv().ok();
        }
        if let Some(status) = process.try_wait()? {
            break Some(status);
        }
        let over = match searching {
            Some(since) => since.elapsed() > limit,
            None => spawned.elapsed() > load_limit,
        };
        if over {
            process.kill()?;
            process.wait()?;
            break None;
        }
        thread::sleep(Duration::from_millis(2));
    };
    let rows = stdout.join().map_err(|_| anyhow!("reading a child"))?;
    let stderr = stderr.join().map_err(|_| anyhow!("reading a child"))?;
    let row = rows.iter().rev().find(|row| row.split(',').count() == TAIL);
    // A child that died after it printed its verdict died in the check of
    // its proof: the row keeps the verdict and says what became of the
    // check. One that died before leaves a row of its own.
    let died = |reason: String| match row {
        Some(row) => {
            let mut fields: Vec<&str> = row.split(',').collect();
            let checked = format!("failed: {reason}");
            fields[CHECKED] = &checked;
            fields.join(",")
        }
        None => {
            let mut fields = vec![String::new(); TAIL];
            fields[2] = "unknown".to_owned();
            fields[3] = reason;
            fields[19] = name(&args.bias);
            fields[20] = forward_copies(args.forward_copies).to_string();
            fields[22] = memory_column(args.memory_limit);
            let since = searching.unwrap_or(spawned);
            fields[9] = format!("{:.3}", since.elapsed().as_secs_f64() * 1000.0);
            fields.join(",")
        }
    };
    Ok(match (status, row) {
        (Some(status), Some(row)) if status.success() => row.clone(),
        (None, _) if searching.is_none() => died("killed while loading".to_owned()),
        (None, _) => died("killed".to_owned()),
        (Some(status), _) => {
            // The row keeps the line that says why; the log gets all of
            // the error output, a panic's place and backtrace with it.
            eprintln!(
                "{} crashed ({status}); its error output:\n{}",
                reference.name,
                stderr.trim_end()
            );
            died(clean(&format!("crash ({status}): {}", cause(&stderr))))
        }
    })
}

/// Returns the line of a dead child's error output that says why it died:
/// the last one that is neither empty nor a note, such as the hint about
/// backtraces that follows a panic's message.
fn cause(stderr: &str) -> &str {
    stderr
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("note:"))
        .unwrap_or("")
}

/// Makes text a CSV field: no commas, no line breaks.
fn clean(text: &str) -> String {
    text.replace(',', ";").replace(['\n', '\r'], " ")
}

/// Runs one problem and prints the tail of its row, on a thread with a
/// stack large enough for the search at its recursion limit and for the
/// parser on a huge input.
pub fn one(args: OneArgs) -> Result<()> {
    let stack = Options::default().stack_size().max(1 << 30);
    let line = thread::Builder::new()
        .stack_size(stack)
        .spawn(move || tail(&args))?
        .join()
        .map_err(|_| anyhow!("the child panicked"))?;
    say(&line);
    Ok(())
}

/// Prints a line for the parent at once.
fn say(line: &str) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}

/// Loads and runs one problem and returns the tail of its row; on the
/// way it says when the problem is loaded, and prints the tail of a
/// proved problem once before the proof is checked.
fn tail(args: &OneArgs) -> String {
    let problem = match problems::load(&args.problem) {
        Ok(problem) => problem,
        Err(error) => return row(&[(2, "error"), (3, &clean(&format!("{error:#}")))]),
    };
    let mode = args.mode.apply(problem.mode);
    let copies = args
        .copies
        .or(problem.copies)
        .unwrap_or(Options::DEFAULT_COPIES);
    let recursion = args
        .recursion_limit
        .unwrap_or(Options::DEFAULT_RECURSION_LIMIT);
    // A known verdict holds in the problem's own mode only: classical
    // linear logic proves more than intuitionistic, affine more than linear.
    let expected = match problem.expected.filter(|_| mode == problem.mode) {
        Some(true) => "provable",
        Some(false) => "unprovable",
        None => "",
    };
    let (occurrences, multiplicity) = match Forest::new(&problem.sequent) {
        Ok(forest) => (forest.len(), multiplicity(&forest)),
        Err(_) => (0, 0),
    };
    let options = Options::default()
        .engine(args.engine.engine())
        .jobs(args.jobs)
        .copies(copies)
        .bias(args.bias.bias())
        .forward_copies(forward_copies(args.forward_copies))
        .memory_limit(memory_limit(args.memory_limit))
        .test_period(args.test_period)
        .recursion_limit(recursion)
        // The check is the child's own, outside the time measured.
        .check(false);
    say(LOADED);

    // The limit is a flag that a thread of its own raises, so that a poll
    // costs one load whatever the engine's cadence: a clock read every so
    // many polls is late by that many, which on a large net, where a poll
    // comes many milliseconds after the last, was seconds.
    let expired = Arc::new(AtomicBool::new(false));
    let limit = Duration::from_secs_f64(args.timeout);
    let (cpu_before, wait_before) = (cpu_ms(), wait_ms());
    let start = Instant::now();
    {
        let expired = Arc::clone(&expired);
        thread::spawn(move || {
            thread::sleep(limit);
            expired.store(true, Ordering::Relaxed);
        });
    }
    let outcome = prove_until(&problem.sequent, mode, &options, || {
        expired.load(Ordering::Relaxed)
    });
    let time = start.elapsed().as_secs_f64() * 1000.0;
    // The time the search took on the CPUs, and the time its thread was
    // ready but waited for one: a run another process slowed down has the
    // latter well above zero.
    let since = |after: Option<f64>, before: Option<f64>| {
        after
            .zip(before)
            .map_or(String::new(), |(a, b)| format!("{:.3}", a - b))
    };
    let (cpu, wait) = (since(cpu_ms(), cpu_before), since(wait_ms(), wait_before));

    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(error) => {
            let verdict = match error {
                Error::NetFragment(_)
                | Error::NetMode(_)
                | Error::EngineMode { .. }
                | Error::NotAdditive { .. }
                | Error::IntuitionisticMix => "refused",
                _ => "error",
            };
            return row(&[
                (0, &copies.to_string()),
                (1, expected),
                (2, verdict),
                (3, &clean(&error.to_string())),
                (7, &occurrences.to_string()),
                (8, &multiplicity.to_string()),
                (16, &recursion.to_string()),
                (19, &name(&args.bias)),
                (20, &forward_copies(args.forward_copies).to_string()),
                (22, &memory_column(args.memory_limit)),
            ]);
        }
    };
    let (verdict, reason) = match &outcome.verdict {
        Verdict::Proved(_) => ("proved", ""),
        Verdict::Unprovable => ("unprovable", ""),
        Verdict::Unknown(reason) => {
            let reason = match reason {
                Reason::Stopped => "timeout",
                Reason::CopyBound(_) => "copy_bound",
                Reason::RecursionLimit => "recursion_limit",
                Reason::MemoryLimit(_) => "memory_limit",
                Reason::IndexLimit => "index_limit",
                _ => "other",
            };
            ("unknown", reason)
        }
    };
    let s = outcome.statistics;
    let tail = |checked: &str, check_ms: &str| {
        [
            copies.to_string(),
            expected.to_owned(),
            verdict.to_owned(),
            reason.to_owned(),
            checked.to_owned(),
            outcome.engine.to_string(),
            outcome.fragment.name_in(mode).to_owned(),
            occurrences.to_string(),
            multiplicity.to_string(),
            format!("{time:.3}"),
            s.nodes.to_string(),
            s.memo_hits.to_string(),
            s.memo_entries.to_string(),
            s.splits.to_string(),
            s.links.to_string(),
            s.tests.to_string(),
            recursion.to_string(),
            cpu.clone(),
            wait.clone(),
            name(&args.bias),
            forward_copies(args.forward_copies).to_string(),
            check_ms.to_owned(),
            memory_column(args.memory_limit),
        ]
        .join(",")
    };
    let Verdict::Proved(proof) = &outcome.verdict else {
        return tail("", "");
    };
    // The verdict is out before the check, which a proof can outgrow.
    say(&tail("", ""));
    let start = Instant::now();
    let checked = match proof.check(mode) {
        Ok(()) => "ok".to_owned(),
        Err(error) => clean(&format!("failed: {error}")),
    };
    let check = start.elapsed().as_secs_f64() * 1000.0;
    tail(&checked, &format!("{check:.3}"))
}

/// A tail with the given fields filled in and the others empty.
fn row(filled: &[(usize, &str)]) -> String {
    let mut fields = vec![""; TAIL];
    for &(i, value) in filled {
        fields[i] = value;
    }
    fields.join(",")
}

/// The CPU time of the process so far, in milliseconds: the user and system
/// time of all its threads, dead ones included, from `/proc/self/stat`, in
/// the kernel's ticks of 10 ms; `None` where there is no such file.
fn cpu_ms() -> Option<f64> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    // After the command in parentheses, utime and stime are the 12th and
    // 13th fields.
    let mut fields = stat.rsplit_once(')')?.1.split_whitespace().skip(11);
    let user: f64 = fields.next()?.parse().ok()?;
    let system: f64 = fields.next()?.parse().ok()?;
    Some((user + system) * 10.0)
}

/// How long the calling thread has been ready to run but waiting for a CPU,
/// in milliseconds, from `/proc/thread-self/schedstat`; `None` where there
/// is no such file.
fn wait_ms() -> Option<f64> {
    let stat = std::fs::read_to_string("/proc/thread-self/schedstat").ok()?;
    let nanoseconds: f64 = stat.split_whitespace().nth(1)?.parse().ok()?;
    Some(nanoseconds / 1e6)
}

/// The most occurrences of one literal, `a` or `~a`, in the forest: what
/// the dispatch compares with its threshold for the net engine.
fn multiplicity(forest: &Forest) -> usize {
    (0..forest.sequent().atom_names().len() as u32)
        .map(|a| {
            let atom = Atom::new(a);
            forest
                .literals(atom, Sign::Var)
                .len()
                .max(forest.literals(atom, Sign::DualVar).len())
        })
        .max()
        .unwrap_or(0)
}
