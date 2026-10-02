// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::{CheckArgs, Format, ProveArgs, SequentFormat};
use crate::io;
use crate::{Status, catch_interrupt, interrupted};
use anyhow::{Context, Result, anyhow, bail};
use linlog::export::svg::{self, Style};
use linlog::export::{Form, latex, rocq, typst};
use linlog::search::{Engine, Options, Outcome, Reason, Statistics, Verdict, prove_until};
use linlog::{Error, Forest, Fragment, Mode, Proof, ProofStructure, Reading, Sequent};
use std::fmt::Write;
use std::thread;
use std::time::{Duration, Instant};

/// How many polls of the stop condition go by between two looks at the
/// clock on one thread, so that the clock costs nothing next to the
/// search; on several threads the driver polls once a millisecond and
/// every poll looks.
const POLLS_PER_CLOCK: u32 = 1024;

/// How many polls go between two looks at the clock with `jobs` threads.
pub(crate) fn polls_per_clock(jobs: usize) -> u32 {
    if jobs > 1 { 1 } else { POLLS_PER_CLOCK }
}

/// Runs `f` on a thread with a stack of `size` bytes, as
/// [`Options::stack_size`] sizes it for the recursion limit, and returns
/// its result.
pub(crate) fn on_large_stack<T: Send>(size: usize, f: impl FnOnce() -> T + Send) -> Result<T> {
    thread::scope(|scope| {
        let handle = thread::Builder::new()
            .name("search".into())
            .stack_size(size)
            .spawn_scoped(scope, f)
            .with_context(|| {
                format!(
                    "cannot start a search thread with a {} MiB stack",
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

/// Returns the derivation of a proof, two-sided in intuitionistic mode:
/// a LaTeX or Typst proof tree in `form` for those formats, an SVG
/// document for SVG, a text tree otherwise; or the checker's complaint
/// with formulas.
pub(crate) fn derivation(proof: &Proof, mode: Mode, format: Format, form: Form) -> Result<String> {
    let derivation = if mode.intuitionistic {
        proof.two_sided_derivation()
    } else {
        proof.derivation()
    };
    let d =
        derivation.map_err(|e| anyhow!("the proof is invalid: {}", e.describe(proof.forest())))?;
    Ok(match format {
        Format::Latex => latex::derivation(&d, form),
        Format::Typst => typst::derivation(&d, form),
        Format::Svg => svg::derivation(&d, &Style::default()),
        Format::Rocq => {
            rocq::derivation(&d, form, &rocq::Options::default()).context("no certificate")?
        }
        Format::Text | Format::Json | Format::Net | Format::NetSvg => d.to_string(),
    })
}

/// Returns a sequent as text: one-sided, or two-sided in intuitionistic
/// mode when it has an intuitionistic reading.
pub fn sequent_text(sequent: &Sequent, mode: Mode) -> Result<String> {
    sequent_in(sequent, mode, SequentFormat::Text, Form::Fragment)
}

/// Returns a sequent in a format, one-sided, or two-sided in
/// intuitionistic mode when it has an intuitionistic reading; LaTeX and
/// Typst in `form`, SVG as a document.
pub fn sequent_in(
    sequent: &Sequent,
    mode: Mode,
    format: SequentFormat,
    form: Form,
) -> Result<String> {
    if !mode.intuitionistic {
        return Ok(match format {
            SequentFormat::Text => sequent.to_string(),
            SequentFormat::Latex => latex::sequent(sequent, form),
            SequentFormat::Typst => typst::sequent(sequent, form),
            SequentFormat::Svg => svg::sequent(sequent, &Style::default()),
        });
    }
    let forest = Forest::new(sequent)?;
    let reading = Reading::new(&forest)
        .map_err(|e| anyhow!("not an intuitionistic sequent: {}", e.describe(&forest)))?;
    Ok(match format {
        SequentFormat::Text => reading.to_string(),
        SequentFormat::Latex => latex::two_sided(&reading, form),
        SequentFormat::Typst => typst::two_sided(&reading, form),
        SequentFormat::Svg => svg::two_sided(&reading, &Style::default()),
    })
}

/// Returns the form `--standalone` asks for, which only the LaTeX, Typst
/// and Rocq formats (`exported`) have: the others have one form, an SVG
/// always being a document, so the flag would change nothing.
pub fn form(standalone: bool, exported: bool) -> Result<Form> {
    match (standalone, exported) {
        (false, _) => Ok(Form::Fragment),
        (true, true) => Ok(Form::Standalone),
        (true, false) => bail!("--standalone needs --format latex, typst or rocq"),
    }
}

/// Returns a line or lines of text as the format writes them next to its
/// output: as they are, or as LaTeX, Typst, XML or Rocq comments.
fn note(format: Format, text: &str) -> String {
    let comment = |line: &str| match format {
        Format::Latex => format!("% {line}"),
        Format::Typst => format!("// {line}"),
        Format::Rocq => format!("(* {line} *)"),
        // An XML comment cannot hold `--`, which flag names bring.
        Format::Svg | Format::NetSvg => format!("<!-- {} -->", line.replace('-', "\u{2010}")),
        Format::Text | Format::Json | Format::Net => line.to_owned(),
    };
    let lines: Vec<String> = text.lines().map(comment).collect();
    lines.join("\n")
}

/// Returns a search error with formulas where the library's message has
/// occurrence ids.
pub(crate) fn describe(error: Error, sequent: &Sequent) -> anyhow::Error {
    match (&error, Forest::new(sequent)) {
        (Error::NotIntuitionistic(e), Ok(forest)) => {
            anyhow!("not an intuitionistic sequent: {}", e.describe(&forest))
        }
        _ => error.into(),
    }
}

/// Returns a proof net as the format writes it: an SVG document for
/// `net-svg`, text otherwise.
fn net_in(net: &ProofStructure, format: Format) -> String {
    match format {
        Format::NetSvg => svg::net(net, &Style::default()),
        _ => net.to_string(),
    }
}

/// Returns the proof net of a proof in a format, or why its links are not
/// one.
fn net(proof: &Proof, mode: Mode, format: Format) -> Result<String> {
    Ok(net_in(
        &ProofStructure::from_proof(proof, mode.mix)?,
        format,
    ))
}

/// Returns the proof net of an outcome in a format: the net the net
/// engine found, or the net of the proof another engine found.
fn net_of(outcome: &Outcome, proof: &Proof, mode: Mode, format: Format) -> Result<String> {
    match &outcome.net {
        Some(found) => Ok(net_in(found, format)),
        None => net(proof, mode, format),
    }
}

/// Fails unless proof nets exist for the sequent in the mode: unit-free
/// MLL, linear, with or without Mix, classical or intuitionistic (where
/// the net is the one of the one-sided sequent).
fn nets_exist(sequent: &Sequent, mode: Mode) -> Result<()> {
    if mode.affine {
        bail!("proof nets exist in linear mode only, with or without --mix, not in {mode} mode");
    }
    let fragment = sequent.fragment();
    if !Fragment::MLL.contains(fragment) {
        return Err(Error::NetFragment(fragment).into());
    }
    Ok(())
}

/// Runs `prove`: reads the sequent, searches on a large stack, and prints
/// the verdict line, the derivation or the proof net and the statistics,
/// or the outcome as JSON.
pub fn prove(args: &ProveArgs) -> Result<Status> {
    let sequent = args.input.sequent()?;
    let mode = args.mode.mode();
    if matches!(args.output.format, Format::Net | Format::NetSvg) {
        nets_exist(&sequent, mode)?;
    }
    let options = Options::default()
        .memo_limit(args.memo_limit)
        .recursion_limit(args.recursion_limit)
        .engine(args.engine.into())
        .fragment(args.fragment.map(Into::into))
        .copies(args.copies)
        .bias(args.bias.into())
        .jobs(if args.deterministic { 1 } else { args.jobs });
    let period = polls_per_clock(if args.deterministic { 1 } else { args.jobs });
    let format = args.output.format;
    let quiet = args.output.quiet;
    let form = form(
        args.output.standalone,
        matches!(format, Format::Latex | Format::Typst | Format::Rocq),
    )?;
    catch_interrupt();

    let (outcome, stop, elapsed, derivation) = on_large_stack(options.stack_size(), || {
        let start = Instant::now();
        let deadline = args.timeout.map(|t| (start + t, t));
        let mut stop = None;
        let mut polls = 0u32;
        let outcome = prove_until(&sequent, mode, &options, || {
            polls = polls.wrapping_add(1);
            if !polls.is_multiple_of(period) {
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
        })
        .map_err(|e| describe(e, &sequent))?;
        let elapsed = start.elapsed();
        let derivation = match (&outcome.verdict, format, quiet) {
            (
                Verdict::Proved(proof),
                Format::Text | Format::Latex | Format::Typst | Format::Svg | Format::Rocq,
                false,
            ) => Some(derivation(proof, mode, format, form)?),
            (Verdict::Proved(proof), Format::Net | Format::NetSvg, false) => {
                Some(net_of(&outcome, proof, mode, format)?)
            }
            _ => None,
        };
        anyhow::Ok((outcome, stop, elapsed, derivation))
    })??;

    let text = match format {
        Format::Json => serde_json::to_string(&outcome)?,
        Format::Text
        | Format::Net
        | Format::Latex
        | Format::Typst
        | Format::Svg
        | Format::NetSvg
        | Format::Rocq => {
            let mut text = note(
                format,
                &verdict_line(&outcome, args.fragment.is_some(), stop),
            );
            if let Some(derivation) = derivation {
                write!(text, "\n{derivation}")?;
            }
            if args.stats {
                let statistics = statistics(outcome.engine, &outcome.statistics, elapsed);
                write!(text, "\n{}", note(format, &statistics))?;
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
        outcome.fragment.name_in(outcome.mode),
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
                (Reason::CopyBound(_), _) => format!("{reason}; raise it with --copies"),
                _ => reason.to_string(),
            };
            format!("unknown ({context}): {why}")
        }
    }
}

/// Returns the statistics as text, one counter per line: the counters the
/// engine that ran keeps.
fn statistics(engine: Engine, s: &Statistics, elapsed: Duration) -> String {
    match engine {
        Engine::Additive => format!(
            "pairs of subformulas visited: {} ({} from the memo)\n\
             memo entries: {}\n\
             time: {elapsed:.2?}",
            s.nodes, s.memo_hits, s.memo_entries
        ),
        Engine::Net => format!(
            "literals chosen: {}\n\
             links tried: {}\n\
             exact tests run: {}\n\
             time: {elapsed:.2?}",
            s.nodes, s.links, s.tests
        ),
        _ => format!(
            "stable sequents visited: {} ({} from the memo)\n\
             memo entries at most: {}\n\
             splits examined: {}\n\
             time: {elapsed:.2?}",
            s.nodes, s.memo_hits, s.memo_entries, s.splits
        ),
    }
}

/// Runs `check`: reads a proof, checks it in the mode the flags give, and
/// prints the verdict and the derivation, or the verdict as JSON.
pub fn check(args: &CheckArgs) -> Result<Status> {
    let text = io::read(args.proof.as_deref(), "proof")?;
    let proof: Proof = serde_json::from_str(&text).context("not a proof in JSON")?;
    let mode = args.mode.mode();
    let format = args.output.format;
    let quiet = args.output.quiet;
    let form = form(
        args.output.standalone,
        matches!(format, Format::Latex | Format::Typst | Format::Rocq),
    )?;
    let (valid, text) = on_large_stack(Options::default().stack_size(), || {
        check_text(&proof, mode, format, form, quiet)
    })??;
    io::write(args.output.output.as_deref(), &text)?;
    Ok(if valid { Status::Yes } else { Status::No })
}

/// Checks the proof and returns whether it is valid, with the output text.
fn check_text(
    proof: &Proof,
    mode: Mode,
    format: Format,
    form: Form,
    quiet: bool,
) -> Result<(bool, String)> {
    let result = proof.check(mode);
    let text = match format {
        Format::Json => serde_json::json!({
            "valid": result.is_ok(),
            "mode": mode,
            "error": result.as_ref().err().map(|e| e.describe(proof.forest()).to_string()),
        })
        .to_string(),
        Format::Text
        | Format::Net
        | Format::Latex
        | Format::Typst
        | Format::Svg
        | Format::NetSvg
        | Format::Rocq => {
            // A sequent with no intuitionistic reading is an invalid proof
            // in intuitionistic mode, printed one-sided.
            let sequent =
                sequent_text(proof.sequent(), mode).unwrap_or_else(|_| proof.sequent().to_string());
            let valid = note(format, &format!("valid proof of {sequent} ({mode})"));
            match &result {
                Ok(()) if quiet => valid,
                Ok(()) if matches!(format, Format::Net | Format::NetSvg) => {
                    nets_exist(proof.sequent(), mode)?;
                    format!("{valid}\n{}", net(proof, mode, format)?)
                }
                Ok(()) => format!("{valid}\n{}", derivation(proof, mode, format, form)?),
                Err(e) => note(
                    format,
                    &format!(
                        "invalid proof of {sequent} ({mode}): {}",
                        e.describe(proof.forest())
                    ),
                ),
            }
        }
    };
    Ok((result.is_ok(), text))
}
