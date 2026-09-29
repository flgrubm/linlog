// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::{CheckArgs, Format, ProveArgs};
use crate::io;
use crate::{Status, interrupted};
use anyhow::{Context, Result, anyhow, bail};
use linlog::search::{Options, Outcome, Reason, Statistics, Verdict, prove_until};
use linlog::{Mode, Proof};
use std::fmt::Write;
use std::thread;
use std::time::{Duration, Instant};

/// The stack one level of recursion may take, in the search or in building
/// and printing the derivation: twice the most the search was measured to
/// take (2 KiB unoptimized, 512 bytes optimized), for the derivation's
/// frames and some slack.
const STACK_PER_LEVEL: usize = if cfg!(debug_assertions) { 4096 } else { 1024 };

/// The smallest stack the search thread gets: a main thread's.
const MIN_STACK: usize = 8 << 20;

/// How many polls of the stop condition go by between two looks at the
/// clock, so that the clock costs nothing next to the search.
const POLLS_PER_CLOCK: u32 = 1024;

/// Runs `f` on a thread whose stack fits `recursion_limit` levels, and
/// returns its result.
fn on_large_stack<T: Send>(recursion_limit: u32, f: impl FnOnce() -> T + Send) -> Result<T> {
    let size = (recursion_limit as usize)
        .saturating_mul(STACK_PER_LEVEL)
        .max(MIN_STACK);
    thread::scope(|scope| {
        let handle = thread::Builder::new()
            .name("search".into())
            .stack_size(size)
            .spawn_scoped(scope, f)
            .with_context(|| {
                format!(
                    "cannot start a search thread with a {} MiB stack for a recursion limit of \
                     {recursion_limit}",
                    size >> 20
                )
            })?;
        Ok(handle
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic)))
    })
}

/// Why the stop condition fired.
#[derive(Clone, Copy)]
enum Stop {
    /// The time limit passed.
    Timeout(Duration),
    /// The user pressed Ctrl-C.
    Interrupt,
}

/// Returns the derivation of a proof as a text tree, or the checker's
/// complaint with formulas.
fn derivation(proof: &Proof) -> Result<String> {
    proof
        .derivation()
        .map(|d| d.to_string())
        .map_err(|e| anyhow!("the proof is invalid: {}", e.describe(proof.forest())))
}

/// Runs `prove`: reads the sequent, searches on a large stack, and prints
/// the verdict line, the derivation and the statistics, or the outcome as
/// JSON.
pub(crate) fn prove(args: &ProveArgs) -> Result<Status> {
    if args.copies.is_some() {
        bail!("--copies bounds the copies of ? formulas, which no engine searches yet");
    }
    let sequent = args.input.sequent()?;
    let mode = args.mode.mode();
    let options = Options::default()
        .memo_limit(args.memo_limit)
        .recursion_limit(args.recursion_limit)
        .engine(args.engine.into())
        .fragment(args.fragment.map(Into::into));
    let format = args.output.format;
    let quiet = args.output.quiet;

    let (outcome, stop, elapsed, derivation) = on_large_stack(args.recursion_limit, || {
        let start = Instant::now();
        let deadline = args.timeout.map(|t| (start + t, t));
        let mut stop = None;
        let mut polls = 0u32;
        let outcome = prove_until(&sequent, mode, &options, || {
            polls = polls.wrapping_add(1);
            if !polls.is_multiple_of(POLLS_PER_CLOCK) {
                return false;
            }
            if interrupted() {
                stop = Some(Stop::Interrupt);
            } else if let Some((deadline, t)) = deadline
                && Instant::now() >= deadline
            {
                stop = Some(Stop::Timeout(t));
            }
            stop.is_some()
        })?;
        let elapsed = start.elapsed();
        let derivation = match (&outcome.verdict, format, quiet) {
            (Verdict::Proved(proof), Format::Text, false) => Some(derivation(proof)?),
            _ => None,
        };
        anyhow::Ok((outcome, stop, elapsed, derivation))
    })??;

    let text = match format {
        Format::Json => serde_json::to_string(&outcome)?,
        Format::Text => {
            let mut text = verdict_line(&outcome, args.fragment.is_some(), stop);
            if let Some(derivation) = derivation {
                write!(text, "\n{derivation}")?;
            }
            if args.stats {
                write!(text, "\n{}", statistics(&outcome.statistics, elapsed))?;
            }
            text
        }
    };
    io::write(args.output.output.as_deref(), &text)?;
    Ok(match outcome.verdict {
        Verdict::Proved(_) => Status::Yes,
        Verdict::Unprovable => Status::No,
        Verdict::Unknown(_) => Status::Unknown,
    })
}

/// Returns the first line of the text output: the verdict, where the
/// search ran, and for an undecided sequent why.
fn verdict_line(outcome: &Outcome, asserted: bool, stop: Option<Stop>) -> String {
    let context = format!(
        "{}{}, {}, {} engine",
        outcome.fragment,
        if asserted { " as asserted" } else { "" },
        outcome.mode,
        outcome.engine
    );
    match &outcome.verdict {
        Verdict::Proved(_) => format!("provable ({context})"),
        Verdict::Unprovable => format!("unprovable ({context}): the search was exhaustive"),
        Verdict::Unknown(reason) => {
            let why = match (reason, stop) {
                (Reason::Stopped, Some(Stop::Timeout(t))) => {
                    format!("the time limit of {t:?} was reached")
                }
                (Reason::Stopped, Some(Stop::Interrupt)) => "interrupted".into(),
                (Reason::RecursionLimit, _) => {
                    format!("{reason}; raise it with --recursion-limit")
                }
                _ => reason.to_string(),
            };
            format!("unknown ({context}): {why}")
        }
    }
}

/// Returns the statistics as text, one counter per line.
fn statistics(s: &Statistics, elapsed: Duration) -> String {
    format!(
        "stable sequents visited: {} ({} from the memo)\n\
         memo entries at most: {}\n\
         splits examined: {}\n\
         time: {elapsed:.2?}",
        s.nodes, s.memo_hits, s.memo_entries, s.splits
    )
}

/// Runs `check`: reads a proof, checks it in the mode the flags give, and
/// prints the verdict and the derivation, or the verdict as JSON.
pub(crate) fn check(args: &CheckArgs) -> Result<Status> {
    let text = io::read(args.proof.as_deref(), "proof")?;
    let proof: Proof = serde_json::from_str(&text).context("not a proof in JSON")?;
    let mode = args.mode.mode();
    let format = args.output.format;
    let quiet = args.output.quiet;
    let (valid, text) = on_large_stack(Options::DEFAULT_RECURSION_LIMIT, || {
        check_text(&proof, mode, format, quiet)
    })??;
    io::write(args.output.output.as_deref(), &text)?;
    Ok(if valid { Status::Yes } else { Status::No })
}

/// Checks the proof and returns whether it is valid, with the output text.
fn check_text(proof: &Proof, mode: Mode, format: Format, quiet: bool) -> Result<(bool, String)> {
    let result = proof.check(mode);
    let text = match format {
        Format::Json => serde_json::json!({
            "valid": result.is_ok(),
            "mode": mode,
            "error": result.as_ref().err().map(|e| e.describe(proof.forest()).to_string()),
        })
        .to_string(),
        Format::Text => {
            let sequent = proof.sequent();
            match &result {
                Ok(()) if quiet => format!("valid proof of {sequent} ({mode})"),
                Ok(()) => format!("valid proof of {sequent} ({mode})\n{}", derivation(proof)?),
                Err(e) => format!(
                    "invalid proof of {sequent} ({mode}): {}",
                    e.describe(proof.forest())
                ),
            }
        }
    };
    Ok((result.is_ok(), text))
}
