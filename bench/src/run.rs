// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Running problems. The parent (`run`) starts this program again as a
//! child (`one`) for every run, so that a crash, a stack overflow or a run
//! that ignores its time limit costs that run only; the child times the
//! search alone, not its start or the parsing, and prints the tail of the
//! run's CSV row. A child that outlives its time limit by more than the
//! grace period is killed.

use crate::problems::{self, Reference, mode_name};
use crate::{OneArgs, RunArgs};
use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;
use linlog::search::{Engine, Options, Reason, Verdict, prove_until};
use linlog::{Atom, Error, Forest, Mode, Sign};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// The columns the parent writes, then those the child prints.
pub const HEADER: &str = "source,family,size,index,problem,mode,engine_requested,jobs,portfolio,\
                          test_period,timeout_s,run,copies,expected,verdict,reason,checked,engine,\
                          fragment,occurrences,multiplicity,time_ms,nodes,memo_hits,memo_entries,\
                          splits,links,tests,recursion_limit";

/// The columns the child prints.
const TAIL: usize = 17;

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
        references.retain(|r| args.only.iter().any(|only| r.name.contains(only.as_str())));
    }
    let jobs = args
        .jobs
        .iter()
        .map(|j| match j.as_str() {
            "all" => Ok(thread::available_parallelism().map_or(1, |n| n.get())),
            n => n.parse().with_context(|| format!("jobs `{n}`")),
        })
        .collect::<Result<Vec<usize>>>()?;

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
                        &args.portfolio.to_string(),
                        &args.test_period.map_or(String::new(), |p| p.to_string()),
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
                            "{},{},{},{},{},{},{},{threads},{},{},{},{run},{tail}",
                            reference.source,
                            reference.family,
                            reference.size.map_or(String::new(), |s| s.to_string()),
                            reference.index.map_or(String::new(), |i| i.to_string()),
                            reference.name,
                            mode_name(mode),
                            name(&engine),
                            args.portfolio,
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
/// source, family, problem, mode, requested engine, jobs, portfolio and
/// test period joined by commas.
fn finished(path: &Path) -> Result<HashSet<String>> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(HashSet::new());
    };
    let mut lines = text.lines();
    let columns: Vec<&str> = lines.next().unwrap_or("").split(',').collect();
    let at = |column: &str| columns.iter().position(|c| *c == column);
    let key = [
        "source",
        "family",
        "problem",
        "mode",
        "engine_requested",
        "jobs",
        "portfolio",
        "test_period",
    ]
    .map(at);
    let Some(key) = key.into_iter().collect::<Option<Vec<usize>>>() else {
        anyhow::bail!("{} is not a CSV file of `run`", path.display());
    };
    Ok(lines
        .map(|line| {
            let fields: Vec<&str> = line.split(',').collect();
            key.iter()
                .map(|&i| fields.get(i).copied().unwrap_or(""))
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
    if args.portfolio {
        command.arg("--portfolio");
    }
    if let Some(copies) = args.copies {
        command.args(["--copies", &copies.to_string()]);
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
    let reader = |mut pipe: Box<dyn Read + Send>| {
        thread::spawn(move || {
            let mut text = String::new();
            let _ = pipe.read_to_string(&mut text);
            text
        })
    };
    let stdout = reader(Box::new(process.stdout.take().expect("piped")));
    let stderr = reader(Box::new(process.stderr.take().expect("piped")));

    let start = Instant::now();
    // Time for the parser, the pool's teardown and the proof check.
    let limit = Duration::from_secs_f64(args.timeout * 1.1 + 5.0);
    let status = loop {
        if let Some(status) = process.try_wait()? {
            break Some(status);
        }
        if start.elapsed() > limit {
            process.kill()?;
            process.wait()?;
            break None;
        }
        thread::sleep(Duration::from_millis(2));
    };
    let stdout = stdout.join().map_err(|_| anyhow!("reading a child"))?;
    let stderr = stderr.join().map_err(|_| anyhow!("reading a child"))?;
    let died = |reason: String| {
        let mut fields = vec![String::new(); TAIL];
        fields[2] = "unknown".to_owned();
        fields[3] = reason;
        fields[9] = format!("{:.3}", start.elapsed().as_secs_f64() * 1000.0);
        fields.join(",")
    };
    Ok(match status {
        None => died("killed".to_owned()),
        Some(status) if status.success() && stdout.trim().split(',').count() == TAIL => {
            stdout.trim().to_owned()
        }
        Some(status) => {
            let last = stderr.lines().rev().find(|l| !l.trim().is_empty());
            died(clean(&format!("crash ({status}): {}", last.unwrap_or(""))))
        }
    })
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
    println!("{line}");
    Ok(())
}

/// Loads and runs one problem and returns the tail of its row.
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
        .portfolio(args.portfolio)
        .copies(copies)
        .test_period(args.test_period)
        .recursion_limit(recursion);

    // The clock is read every 64 polls on one thread, where the engine
    // polls millions of times a second, and every poll on a pool, whose
    // driver polls once a millisecond.
    let every = if args.jobs > 1 { 1 } else { 64 };
    let mut polls = 0u64;
    let start = Instant::now();
    let deadline = start + Duration::from_secs_f64(args.timeout);
    let outcome = prove_until(&problem.sequent, mode, &options, || {
        polls += 1;
        polls.is_multiple_of(every) && Instant::now() >= deadline
    });
    let time = start.elapsed().as_secs_f64() * 1000.0;

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
            ]);
        }
    };
    let (verdict, reason, checked) = match &outcome.verdict {
        Verdict::Proved(proof) => {
            let checked = match proof.check(mode) {
                Ok(()) => "ok".to_owned(),
                Err(error) => clean(&format!("failed: {error}")),
            };
            ("proved", "", checked)
        }
        Verdict::Unprovable => ("unprovable", "", String::new()),
        Verdict::Unknown(reason) => {
            let reason = match reason {
                Reason::Stopped => "timeout",
                Reason::CopyBound(_) => "copy_bound",
                Reason::RecursionLimit => "recursion_limit",
                Reason::ContextTooWide(_) => "context_too_wide",
                _ => "other",
            };
            ("unknown", reason, String::new())
        }
    };
    let s = outcome.statistics;
    [
        copies.to_string(),
        expected.to_owned(),
        verdict.to_owned(),
        reason.to_owned(),
        checked,
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
    ]
    .join(",")
}

/// A tail with the given fields filled in and the others empty.
fn row(filled: &[(usize, &str)]) -> String {
    let mut fields = vec![""; TAIL];
    for &(i, value) in filled {
        fields[i] = value;
    }
    fields.join(",")
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
