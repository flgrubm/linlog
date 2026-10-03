// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::{CheckArgs, Format, OutputArgs, ProveArgs, SequentFormat};
use crate::io;
use crate::{Status, catch_interrupt, interrupted};
use anyhow::{Context, Result, anyhow, bail};
use linlog::export::svg::{self, Style};
use linlog::export::{Form, latex, rocq, typst};
use linlog::search::{Engine, Options, Outcome, Reason, Statistics, Verdict, prove_until};
use linlog::{
    Error, Forest, Fragment, Mode, Proof, ProofStructure, Reading, Sequent, Size, ViewError,
    ViewOptions,
};
use std::fmt::Write;
use std::io::IsTerminal;
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

/// How many inferences are built, or pieces of text written, between two
/// looks at the clock and the Ctrl-C flag while a derivation is made.
const STEPS_PER_CLOCK: u32 = 256;

/// How a derivation is to be shown: what the output arguments ask for and
/// where the output goes.
pub(crate) struct Show {
    /// The format.
    format: Format,
    /// The form of a LaTeX, Typst or Rocq derivation.
    form: Form,
    /// The bound on what is built.
    view: ViewOptions,
    /// Whether the output goes to a terminal.
    terminal: bool,
}

impl Show {
    /// Reads the output arguments; `exported` says the format has a
    /// document form.
    pub(crate) fn new(output: &OutputArgs) -> Result<Self> {
        let format = output.format;
        let form = form(
            output.standalone,
            matches!(format, Format::Latex | Format::Typst | Format::Rocq),
        )?;
        let terminal = output.output.is_none() && std::io::stdout().is_terminal();
        Ok(Self {
            format,
            form,
            view: output.derivation_limit.into(),
            terminal,
        })
    }

    /// The text tree of any size within `view`, wherever it goes: what a
    /// session prints when asked for a proof.
    pub(crate) fn text(view: ViewOptions) -> Self {
        Self {
            format: Format::Text,
            form: Form::Fragment,
            view,
            terminal: false,
        }
    }

    /// Writes a line about a derivation that was left out: after the
    /// verdict when the output is a terminal, to standard error otherwise,
    /// so that a file or a pipe gets what it would get from a small proof.
    fn left_out(&self, text: &mut String, line: &str) {
        if self.terminal {
            text.push('\n');
            text.push_str(&note(self.format, line));
        } else {
            eprintln!("{line}");
        }
    }
}

/// What became of the derivation of a proof.
pub(crate) enum Shown {
    /// It was written, as this text.
    Written(String),
    /// It was left out, for the reason this line gives with the ways to
    /// get it.
    LeftOut(String),
    /// It was not asked for.
    Nothing,
}

/// A text that takes what is written to it until a stop condition fires.
struct Halting<'a> {
    /// What was written.
    text: String,
    /// The stop condition.
    halt: &'a mut dyn FnMut() -> bool,
}

impl Write for Halting<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if (self.halt)() {
            return Err(std::fmt::Error);
        }
        self.text.push_str(s);
        Ok(())
    }
}

/// Returns a number of bytes in the largest binary unit that leaves it at
/// least one.
pub(crate) fn bytes_text(bytes: u64) -> String {
    if bytes == u64::MAX {
        return "more than 16 EiB".to_owned();
    }
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let (mut value, mut unit) = (bytes as f64, 0);
    while value >= 1024.0 && unit + 1 < units.len() {
        (value, unit) = (value / 1024.0, unit + 1);
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", units[unit])
    }
}

/// Returns the line for a derivation past the limit.
fn too_large(size: &Size, limit: u64) -> String {
    format!(
        "the derivation is not written: its {} inferences with {} characters of sequents are \
         estimated at {}, over the limit of {}; --format json writes the proof itself, \
         --derivation-limit SIZE raises the limit and --derivation-limit none lifts it",
        size.inferences,
        size.characters,
        bytes_text(size.bytes()),
        bytes_text(limit)
    )
}

/// Makes the derivation of a proof as `show` asks, two-sided in
/// intuitionistic mode: a LaTeX or Typst proof tree, a Rocq script, an
/// SVG document or a text tree; or says why it is left out: a derivation
/// past the limit, or one that `halt` stopped, which is polled as it is built and written and
/// whose reason `why` then gives. Fails with the checker's complaint,
/// with formulas, on a proof that is none.
pub(crate) fn derivation(
    proof: &Proof,
    mode: Mode,
    show: &Show,
    mut halt: impl FnMut() -> bool,
    why: impl Fn() -> String,
) -> Result<Shown> {
    let invalid =
        |e: linlog::CheckError| anyhow!("the proof is invalid: {}", e.describe(proof.forest()));
    let stopped = || Shown::LeftOut(format!("the derivation is not written: {}", why()));
    let built = if mode.intuitionistic {
        proof.two_sided_derivation_with(&show.view, &mut halt)
    } else {
        proof.derivation_with(&show.view, &mut halt)
    };
    let d = match built {
        Ok(d) => d,
        Err(ViewError::Invalid(e)) => return Err(invalid(e)),
        Err(ViewError::TooLarge { size, limit }) => {
            return Ok(Shown::LeftOut(too_large(&size, limit)));
        }
        Err(ViewError::Stopped) => return Ok(stopped()),
    };
    Ok(Shown::Written(match show.format {
        Format::Latex => latex::derivation(&d, show.form),
        Format::Typst => typst::derivation(&d, show.form),
        Format::Svg => svg::derivation(&d, &Style::default()),
        Format::Rocq => {
            rocq::derivation(&d, show.form, &rocq::Options::default()).context("no certificate")?
        }
        Format::Text | Format::Json | Format::Net | Format::NetSvg => {
            let mut out = Halting {
                text: String::new(),
                halt: &mut halt,
            };
            if write!(out, "{d}").is_err() {
                return Ok(stopped());
            }
            out.text
        }
    }))
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
        .forward_copies(args.forward_copies)
        .check(!args.no_check)
        .jobs(if args.deterministic { 1 } else { args.jobs });
    let period = polls_per_clock(if args.deterministic { 1 } else { args.jobs });
    let format = args.output.format;
    let quiet = args.output.quiet;
    let show = Show::new(&args.output)?;
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
        // The time limit and Ctrl-C hold for the derivation as for the
        // search.
        let over =
            || interrupted() || deadline.is_some_and(|(deadline, _)| Instant::now() >= deadline);
        let mut steps = 0u32;
        let halt = || {
            steps = steps.wrapping_add(1);
            steps.is_multiple_of(STEPS_PER_CLOCK) && over()
        };
        let why = || match deadline {
            _ if interrupted() => "interrupted".to_owned(),
            Some((_, t)) => format!("the time limit of {t:?} was reached"),
            None => "stopped".to_owned(),
        };
        let derivation = match (&outcome.verdict, format, quiet) {
            (
                Verdict::Proved(proof),
                Format::Text | Format::Latex | Format::Typst | Format::Svg | Format::Rocq,
                false,
            ) if over() => {
                let _ = proof;
                Shown::LeftOut(format!("the derivation is not written: {}", why()))
            }
            (
                Verdict::Proved(proof),
                Format::Text | Format::Latex | Format::Typst | Format::Svg | Format::Rocq,
                false,
            ) => derivation(proof, mode, &show, halt, why)?,
            (Verdict::Proved(proof), Format::Net | Format::NetSvg, false) => {
                Shown::Written(net_of(&outcome, proof, mode, format)?)
            }
            _ => Shown::Nothing,
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
            match derivation {
                Shown::Written(derivation) => write!(text, "\n{derivation}")?,
                Shown::LeftOut(line) => show.left_out(&mut text, &line),
                Shown::Nothing => {}
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
    let quiet = args.output.quiet;
    let show = Show::new(&args.output)?;
    let (valid, text) = on_large_stack(Options::default().stack_size(), || {
        check_text(&proof, mode, &show, quiet)
    })??;
    io::write(args.output.output.as_deref(), &text)?;
    Ok(if valid { Status::Yes } else { Status::No })
}

/// Checks the proof and returns whether it is valid, with the output text.
fn check_text(proof: &Proof, mode: Mode, show: &Show, quiet: bool) -> Result<(bool, String)> {
    let format = show.format;
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
                Ok(()) => {
                    let mut text = valid;
                    let stopped = || "stopped".to_owned();
                    match derivation(proof, mode, show, || false, stopped)? {
                        Shown::Written(derivation) => write!(text, "\n{derivation}")?,
                        Shown::LeftOut(line) => show.left_out(&mut text, &line),
                        Shown::Nothing => {}
                    }
                    text
                }
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
