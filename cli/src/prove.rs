// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::{
    CheckArgs, Format, OutputArgs, ProveArgs, SequentFormat, Threads, Tree, threads,
};
use crate::io;
use crate::limit::{Deadline, Notice};
use crate::{Status, catch_interrupt, interrupted};
use anyhow::{Context, Result, anyhow, bail};
use linlog::export::svg::{self, Style};
use linlog::export::{Form, latex, rocq, typst};
use linlog::search::{Engine, Options, Outcome, Reason, Verdict, prove_goal};
use linlog::{
    Error, Forest, Fragment, Mode, Proof, ProofStructure, Reading, Sequent, Size, ViewError,
    ViewOptions,
};
use std::fmt::Write;
use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

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

/// Runs a search with the threads `threads` gives: from the start, or on
/// one thread first and, if that has not decided when `threads.alone` has
/// passed, with a pool of the other threads beside it, the first of the
/// two to decide answering. The single thread is not stopped when the
/// pool starts: a pool may search worse than one thread, and what one
/// thread decides within the limit stays decided. Each is the search it
/// would be alone, within the memory bound of the options: a halved bound
/// starves the memo of a wide sequent, whose every entry is large, and
/// the search with it. `halt` is the command's stop condition, and
/// `search` runs a search with the options and stop condition given. The
/// outcome of two searches has the counters of both.
pub(crate) fn alone_first<E: Send>(
    options: &Options,
    threads: Threads,
    halt: &(dyn Fn() -> bool + Sync),
    search: impl Fn(&Options, &mut dyn FnMut() -> bool) -> Result<Outcome, E> + Sync,
) -> Result<Outcome, E> {
    let Some(alone) = threads.alone else {
        return search(options, &mut || halt());
    };
    // A pool of one thread would be the single thread's search again.
    let pool = threads.jobs.saturating_sub(1).max(2);
    let decided = AtomicBool::new(false);
    let is_decided =
        |o: &Result<Outcome, E>| matches!(o, Ok(o) if !matches!(o.verdict, Verdict::Unknown(_)));
    thread::scope(|scope| {
        let (done, finished) = std::sync::mpsc::channel::<()>();
        let (search, decided, is_decided) = (&search, &decided, &is_decided);
        let single = thread::Builder::new()
            .name("search alone".into())
            .stack_size(options.stack_size())
            .spawn_scoped(scope, move || {
                let outcome = search(&options.clone().jobs(1), &mut || {
                    halt() || decided.load(Ordering::Relaxed)
                });
                if is_decided(&outcome) {
                    decided.store(true, Ordering::Relaxed);
                }
                let _ = done.send(());
                outcome
            });
        // Without a second thread the search runs on this one alone.
        let Ok(single) = single else {
            return search(&options.clone().jobs(1), &mut || halt());
        };
        let join = |single: thread::ScopedJoinHandle<'_, Result<Outcome, E>>| {
            single
                .join()
                .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
        };
        if finished.recv_timeout(alone).is_ok() || halt() {
            return join(single);
        }
        let pooled = search(&options.clone().jobs(pool), &mut || {
            halt() || decided.load(Ordering::Relaxed)
        });
        if is_decided(&pooled) {
            decided.store(true, Ordering::Relaxed);
        }
        let first = join(single);
        let (mut outcome, other) = if is_decided(&first) {
            (first?, pooled?)
        } else {
            (pooled?, first?)
        };
        let (s, f) = (&mut outcome.statistics, &other.statistics);
        s.nodes += f.nodes;
        s.memo_hits += f.memo_hits;
        s.memo_entries = s.memo_entries.max(f.memo_entries);
        s.splits += f.splits;
        s.links += f.links;
        s.tests += f.tests;
        s.copies = s.copies.max(f.copies);
        Ok(outcome)
    })
}

/// How long a search runs before a line on standard error says that it
/// does, when standard error is a terminal.
pub(crate) const NOTICE_AFTER: Duration = Duration::from_millis(500);

/// Returns the line on standard error while a search runs long: how long
/// it may run, whether it deepens its copy bound, and how to change that.
pub(crate) fn notice_line(limit: Option<Duration>, deepens: bool) -> String {
    let deepening = if deepens {
        ", deepening the copy bound"
    } else {
        ""
    };
    match limit {
        Some(t) => format!("searching for at most {t:?}{deepening}; --timeout changes the limit"),
        None => format!("searching without a time limit{deepening}; Ctrl-C stops it"),
    }
}

/// Returns why the command's stop condition fires, if it does: Ctrl-C
/// or the deadline.
pub(crate) fn stopped(deadline: &Deadline) -> Option<Stop> {
    if interrupted() {
        Some(Stop::Interrupt)
    } else if deadline.passed() {
        deadline.limit().map(Stop::Timeout)
    } else {
        None
    }
}

/// Why the stop condition fired.
#[derive(Clone, Copy)]
pub(crate) enum Stop {
    /// The time limit passed.
    Timeout(Duration),
    /// The user pressed Ctrl-C.
    Interrupt,
}

/// How many screens of lines a proof tree may fill and still be printed
/// on a terminal without being asked for.
const SCREENS: u64 = 3;

/// How many inferences are built, or pieces of text written, between two
/// looks at the time limit and the Ctrl-C flag while a derivation is made.
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
    /// When the text tree is printed.
    tree: Tree,
    /// The columns and rows of the terminal the output goes to, if it goes
    /// to one.
    terminal: Option<(u64, u64)>,
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
        // The size of the terminal that standard output is, and of no
        // other stream's.
        let stdout = std::io::stdout();
        let terminal = match (&output.output, stdout.is_terminal()) {
            (None, true) => Some(
                terminal_size::terminal_size_of(stdout)
                    .map_or((80, 24), |(w, h)| (u64::from(w.0), u64::from(h.0))),
            ),
            _ => None,
        };
        Ok(Self {
            format,
            form,
            view: output.derivation_limit.into(),
            tree: output.tree,
            terminal,
        })
    }

    /// The same, with every derivation and the check behind it within
    /// `memory` bytes.
    pub(crate) fn within(mut self, memory: Option<u64>) -> Self {
        self.view.memory = memory;
        self
    }

    /// The text tree of any size within `view`, wherever it goes: what a
    /// session prints when asked for a proof.
    pub(crate) fn text(view: ViewOptions) -> Self {
        Self {
            format: Format::Text,
            form: Form::Fragment,
            view,
            tree: Tree::Always,
            terminal: None,
        }
    }

    /// The columns and lines a text tree may take to be printed unasked,
    /// when the output is a terminal and the switch leaves it to the fit.
    fn fit(&self) -> Option<(u64, u64)> {
        match (self.tree, self.format, self.terminal) {
            (Tree::Auto, Format::Text, Some((columns, rows))) => {
                Some((columns, rows.saturating_mul(SCREENS)))
            }
            _ => None,
        }
    }

    /// Writes a line about a derivation that was left out: after the
    /// verdict when the output is a terminal, to standard error otherwise,
    /// so that a file or a pipe gets what it would get from a small proof.
    fn left_out(&self, text: &mut String, line: &str) {
        if self.terminal.is_some() {
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

/// Returns a count of a derivation's size as text; one that reached the
/// most its counter holds is only known to be beyond it.
pub(crate) fn count_text(count: u64) -> String {
    if count == u64::MAX {
        "more than 10¹⁹".to_owned()
    } else {
        count.to_string()
    }
}

/// Returns the line for a tree that does not fit the terminal.
fn unfit(inferences: u64, width: &str, lines: u64, columns: u64, most: u64) -> String {
    format!(
        "the proof tree is not shown: {} inferences, {width} columns by {} lines, for a \
         terminal of {columns} columns and at most {most} lines ({SCREENS} screens); print \
         it with --tree always, write it with --output FILE, or get the proof with --format \
         json",
        count_text(inferences),
        count_text(lines)
    )
}

/// Returns the line for a derivation past the limit.
fn too_large(size: &Size, limit: u64) -> String {
    format!(
        "the derivation is not written: its {} inferences with {} characters of sequents are \
         estimated at {}, over the limit of {}; --format json writes the proof itself, \
         --derivation-limit SIZE raises the limit and --derivation-limit none lifts it",
        count_text(size.inferences),
        count_text(size.characters),
        bytes_text(size.bytes()),
        bytes_text(limit)
    )
}

/// Returns the line for a derivation whose proof could not be read
/// within the memory limit.
fn proof_unread(limit: u64) -> String {
    format!(
        "the derivation is not written: reading the proof for it takes more than the memory \
         limit of {}; --format json writes the proof itself, and --memory-limit SIZE raises \
         the limit",
        bytes_text(limit)
    )
}

/// Returns the line for a derivation past the memory limit.
fn over_memory(size: &Size, limit: u64) -> String {
    format!(
        "the derivation is not written: its {} inferences with {} characters of sequents are \
         estimated at {}, over the memory limit of {}; --format json writes the proof itself, \
         and --memory-limit SIZE raises the limit",
        count_text(size.inferences),
        count_text(size.characters),
        bytes_text(size.bytes()),
        bytes_text(limit)
    )
}

/// Makes the derivation of a proof as `show` asks, two-sided in
/// intuitionistic mode: a LaTeX or Typst proof tree, a Rocq script, an
/// SVG document or a text tree; or says why it is left out: a text tree
/// that does not fit the terminal, a derivation past the limit, or one
/// that `halt` stopped, which is polled as it is built and written and
/// whose reason `why` then gives. Fails with the checker's complaint,
/// with formulas, on a proof that is none.
pub(crate) fn derivation(
    proof: &Proof,
    mode: Mode,
    show: &Show,
    mut halt: impl FnMut() -> bool,
    why: impl Fn() -> String,
) -> Result<Shown> {
    if show.tree == Tree::Never {
        return Ok(Shown::Nothing);
    }
    let invalid =
        |e: linlog::CheckError| anyhow!("the proof is invalid: {}", e.describe(proof.forest()));
    let stopped = || Shown::LeftOut(format!("the derivation is not written: {}", why()));
    // A tree that cannot fit is known from its size alone, before
    // anything is built.
    let fit = show.fit();
    if let Some((columns, most)) = fit {
        let size = match proof.derivation_size_within(mode.intuitionistic, show.view.memory) {
            Ok(size) => size,
            // No verdict on the proof: the pass was given up.
            Err(e) if e.is_refusal() => {
                return Ok(Shown::LeftOut(proof_unread(show.view.memory.unwrap_or(0))));
            }
            Err(e) => return Err(invalid(e)),
        };
        if size.width > columns || size.lines() > most {
            let width = format!("at least {}", size.width);
            return Ok(Shown::LeftOut(unfit(
                size.inferences,
                &width,
                size.lines(),
                columns,
                most,
            )));
        }
    }
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
        Err(ViewError::Memory {
            size: Some(size),
            limit,
        }) => return Ok(Shown::LeftOut(over_memory(&size, limit))),
        Err(ViewError::Memory { size: None, limit }) => {
            return Ok(Shown::LeftOut(proof_unread(limit)));
        }
        Err(ViewError::Stopped) => return Ok(stopped()),
        // Any other bound of the view's: the error says which.
        Err(error) => return Ok(Shown::LeftOut(error.to_string())),
    };
    if let Some((columns, most)) = fit {
        let (width, lines) = d.text_size();
        let (width, lines) = (width as u64, lines as u64);
        if width > columns || lines > most {
            let inferences = d.inferences().len() as u64;
            return Ok(Shown::LeftOut(unfit(
                inferences,
                &width.to_string(),
                lines,
                columns,
                most,
            )));
        }
    }
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
    // The sequent was admitted when it was read.
    let forest = Forest::within(sequent, u64::MAX)?;
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
    match (&error, Forest::within(sequent, u64::MAX)) {
        (Error::NotIntuitionistic(e), Ok(forest)) => {
            anyhow!("not an intuitionistic sequent: {}", e.describe(&forest))
        }
        (Error::Unchecked(_), _) => anyhow!(
            "the search found a proof, but {error}; raise the limit with --memory-limit, or take \
             the proof unchecked with --no-check"
        ),
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

/// What `prove` answers when the time limit passed before the sequent was
/// read: the verdict is unknown, and the line names no fragment and no
/// engine, since nothing is known of the sequent. The line is the output,
/// as a comment of the format; JSON has no form for it, so there it goes
/// to standard error.
fn unread(args: &ProveArgs, limit: Duration) -> Result<Status> {
    let line =
        format!("unknown: the time limit of {limit:?} was reached while the sequent was read");
    match args.output.format {
        Format::Json => eprintln!("{line}"),
        format => io::write(args.output.output.as_deref(), &note(format, &line))?,
    }
    Ok(Status::Unknown)
}

/// Runs `prove`: reads the sequent, searches on a large stack, and prints
/// the verdict line, the derivation or the proof net and the statistics,
/// or the outcome as JSON. A time limit counts from here: the sequent is
/// read, parsed and laid out as a forest under it, on a thread the
/// command stops waiting for when the limit passes.
pub fn prove(args: &ProveArgs) -> Result<Status> {
    let deadline = Deadline::start(args.timeout.0, Instant::now())?;
    let input = args.input.clone();
    let loaded = deadline.within(move || input.forest())?;
    let Some(forest) = loaded else {
        let limit = deadline.limit().expect("only a limit passes");
        return unread(args, limit);
    };
    let forest = forest?;
    let sequent = forest.sequent();
    let mode = args.mode.mode();
    if matches!(args.output.format, Format::Net | Format::NetSvg) {
        nets_exist(sequent, mode)?;
    }
    let options = Options::default()
        .memo_limit(args.memo_limit)
        .recursion_limit(args.recursion_limit)
        .engine(args.engine.into())
        .fragment(args.fragment.map(Into::into))
        .copies(args.copies.0)
        .bias(args.bias.into())
        .forward_copies(args.forward_copies)
        .check(!args.no_check)
        .memory_limit(args.memory_limit.0)
        .occurrence_limit(args.input.most());
    let threads = threads(args.jobs, args.pool_after, args.deterministic);
    let options = options.jobs(threads.jobs);
    let deepens = args.copies.0.is_none() && sequent.fragment().has_exponentials();
    let format = args.output.format;
    let quiet = args.output.quiet;
    let show = Show::new(&args.output)?.within(args.memory_limit.0);
    catch_interrupt();

    let (outcome, stop, elapsed, derivation) = on_large_stack(options.stack_size(), || {
        let start = Instant::now();
        let notice = Notice::start(NOTICE_AFTER, notice_line(deadline.limit(), deepens));
        // Both conditions are flags, so every poll asks both.
        let halt = || interrupted() || deadline.passed();
        let outcome = alone_first(&options, threads, &halt, |options, halt| {
            prove_goal(&forest, forest.roots(), mode, options, halt)
        })
        .map_err(|e| describe(e, sequent))?;
        let stop = stopped(&deadline);
        drop(notice);
        let elapsed = start.elapsed();
        // The time limit and Ctrl-C hold for the derivation as for the
        // search.
        let over = || interrupted() || deadline.passed();
        let mut steps = 0u32;
        let halt = || {
            steps = steps.wrapping_add(1);
            steps.is_multiple_of(STEPS_PER_CLOCK) && over()
        };
        let why = || match deadline.limit() {
            _ if interrupted() => "interrupted".to_owned(),
            Some(t) => format!("the time limit of {t:?} was reached"),
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
            let ended = Ended {
                stop,
                elapsed,
                recursion_limit: args.recursion_limit,
            };
            let mut text = note(
                format,
                &verdict_line(&outcome, args.fragment.is_some(), &ended),
            );
            match derivation {
                Shown::Written(derivation) => write!(text, "\n{derivation}")?,
                Shown::LeftOut(line) => show.left_out(&mut text, &line),
                Shown::Nothing => {}
            }
            if args.stats {
                let statistics = statistics(&outcome, elapsed);
                write!(text, "\n{}", note(format, &statistics))?;
            }
            text
        }
    };
    io::write(args.output.output.as_deref(), &text)?;
    Ok(match outcome.verdict {
        Verdict::Proved(_) => Status::Yes,
        Verdict::Unprovable(_) => Status::No,
        Verdict::Unknown(_) => Status::Unknown,
    })
}

/// How a search ended, as far as the command knows it beyond the outcome.
pub(crate) struct Ended {
    /// Why the command's stop condition fired, if it did.
    pub(crate) stop: Option<Stop>,
    /// How long the search took.
    pub(crate) elapsed: Duration,
    /// The recursion limit it ran under.
    pub(crate) recursion_limit: u32,
}

/// Returns the first line of the text output: the verdict, where the
/// search ran, and for an undecided sequent why.
fn verdict_line(outcome: &Outcome, asserted: bool, ended: &Ended) -> String {
    let context = format!(
        "{}{}, {}, {} engine",
        outcome.fragment.name_in(outcome.mode),
        if asserted { " as asserted" } else { "" },
        outcome.mode,
        outcome.engine
    );
    match &outcome.verdict {
        Verdict::Proved(_) => format!("provable ({context})"),
        Verdict::Unprovable(refutation) => format!("unprovable ({context}): {refutation}"),
        Verdict::Unknown(reason) => {
            format!("unknown ({context}): {}", unknown(*reason, outcome, ended))
        }
    }
}

/// Returns why a search did not decide, for each way it can end: the
/// bound or limit it reached, after how long, at which copy bound when it
/// deepened one, and the flag that changes it.
pub(crate) fn unknown(reason: Reason, outcome: &Outcome, ended: &Ended) -> String {
    let deepened = outcome.fragment.has_exponentials()
        && matches!(outcome.engine, Engine::Focus | Engine::TwoSided);
    let at = if deepened {
        format!(" at a copy bound of {}", outcome.statistics.copies)
    } else {
        String::new()
    };
    let after = format!("after {:.2?}", ended.elapsed);
    match (reason, ended.stop) {
        (Reason::Stopped, Some(Stop::Timeout(t))) => {
            format!(
                "the time limit of {t:?} was reached{at}; --timeout DURATION gives the search longer"
            )
        }
        (Reason::Stopped, Some(Stop::Interrupt)) => format!("interrupted {after}{at}"),
        (Reason::CopyBound(_), _) => format!(
            "{reason} {after}; raise it with --copies N, or lift it with --copies none to deepen \
             it while the time limit lasts"
        ),
        (Reason::RecursionLimit, _) => format!(
            "the recursion limit of {} was reached {after}{at}; raise it with --recursion-limit N",
            ended.recursion_limit
        ),
        (Reason::MemoryLimit(_), _) => {
            format!("{reason} {after}{at}; raise it with --memory-limit SIZE")
        }
        // Any other reason, which no flag changes.
        _ => format!("{reason} {after}{at}"),
    }
}

/// Returns the statistics as text, one counter per line: the counters the
/// engine that ran keeps, and the copy bound it reached where the
/// fragment has exponentials.
fn statistics(outcome: &Outcome, elapsed: Duration) -> String {
    let s = &outcome.statistics;
    match outcome.engine {
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
             splits examined: {}\n{}\
             time: {elapsed:.2?}",
            s.nodes,
            s.memo_hits,
            s.memo_entries,
            s.splits,
            if outcome.fragment.has_exponentials() {
                format!("copy bound reached: {}\n", s.copies)
            } else {
                String::new()
            }
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
    let show = Show::new(&args.output)?.within(args.memory_limit.0);
    let (valid, text) = on_large_stack(Options::default().stack_size(), || {
        check_text(&proof, mode, &show, quiet)
    })??;
    io::write(args.output.output.as_deref(), &text)?;
    Ok(if valid { Status::Yes } else { Status::No })
}

/// Checks the proof and returns whether it is valid, with the output text.
fn check_text(proof: &Proof, mode: Mode, show: &Show, quiet: bool) -> Result<(bool, String)> {
    let format = show.format;
    let result = proof.check_within(mode, show.view.memory);
    if let Err(refusal) = &result
        && refusal.is_refusal()
    {
        bail!("the proof is not checked: {refusal}; raise the limit with --memory-limit");
    }
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
