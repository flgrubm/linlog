// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::InteractArgs;
use crate::limit::Deadline;
use crate::prove::{Show, Shown, bytes_text, count_text, derivation, describe, on_large_stack};
use crate::{Status, catch_interrupt, clear_interrupt, interrupted, io};
use anyhow::{Context, Result, bail};
use linlog::export::svg::{self, Style};
use linlog::export::{Form, latex, typst};
use linlog::search::{Options, Outcome, Reason, Verdict};
use linlog::{Error, InfId, Interactive, Position, Reading, Rule, ViewError, ViewOptions};
use std::fmt::Write as _;
use std::io::{BufRead, IsTerminal, Write};
use std::path::Path;
use std::time::{Duration, Instant};

/// The commands, as `help` prints them.
const HELP: &str = "\
goals               the open goals, with the positions of their formulas
rules G P           the rules that act on formula P of goal G
apply G P RULE [P…] apply a rule; further positions go to the left premise of a ⊗ or Mix
undo                retract the last step
close [G]           let the search close goal G, or every open goal
show [latex|typst|svg]
                    the derivation so far, as text, as a LaTeX or Typst proof tree, or as SVG
proof [FILE]        check the finished proof and print it, or write it as JSON
save FILE           write the session as JSON
load FILE           resume a session written by save
help                this list
quit                end the session";

/// Runs `interact`: starts or resumes a session, then reads commands from
/// standard input until `quit` or the end of input, on a thread whose stack
/// fits the recursion limit.
pub fn interact(args: &InteractArgs) -> Result<Status> {
    let state = match &args.state {
        Some(path) => load(path)?,
        None => {
            if args.input.sequent.is_none() && args.input.file.is_none() {
                bail!(
                    "no sequent given: pass it as an argument or with --file, since standard input carries the commands"
                );
            }
            let sequent = args.input.sequent()?;
            Interactive::new(&sequent, args.mode.mode()).map_err(|e| describe(e, &sequent))?
        }
    };
    let options = Options::default()
        .memo_limit(args.memo_limit)
        .recursion_limit(args.recursion_limit)
        .copies(args.copies)
        .bias(args.bias.into())
        .forward_copies(args.forward_copies)
        .jobs(if args.deterministic { 1 } else { args.jobs });
    catch_interrupt();
    let stack_size = options.stack_size();
    let mut session = Session {
        state,
        options,
        view: args.derivation_limit.into(),
        timeout: args.timeout,
    };
    on_large_stack(stack_size, move || session.run())?
}

/// Reads a session from a JSON file.
fn load(path: &Path) -> Result<Interactive> {
    let text = io::read(Some(path), "session")?;
    serde_json::from_str(&text)
        .with_context(|| format!("{} is not a saved session", path.display()))
}

/// A running session: the state and the search settings.
struct Session {
    /// The proof in progress.
    state: Interactive,
    /// The search settings of `close`.
    options: Options,
    /// The bound on the derivation `close` grafts.
    view: ViewOptions,
    /// How long a `close` may take.
    timeout: Option<Duration>,
}

impl Session {
    /// Reads and runs commands until `quit` or the end of input, and returns
    /// whether the proof was finished and checks.
    fn run(&mut self) -> Result<Status> {
        let stdin = std::io::stdin();
        let prompt = stdin.is_terminal();
        let mut stdout = std::io::stdout();
        let mut lines = stdin.lock().lines();
        loop {
            if prompt {
                write!(stdout, "> ")?;
                stdout.flush()?;
            }
            let Some(line) = lines.next() else {
                break;
            };
            let line = line.context("cannot read standard input")?;
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.first() == Some(&"quit") {
                break;
            }
            let text = match self.command(&words) {
                Ok(text) => text,
                Err(e) => format!("error: {e:#}"),
            };
            if !text.is_empty() {
                writeln!(stdout, "{text}")?;
            }
        }
        Ok(if self.state.is_complete() && self.state.proof().is_ok() {
            Status::Yes
        } else {
            Status::No
        })
    }

    /// Runs one command and returns what to print.
    fn command(&mut self, words: &[&str]) -> Result<String> {
        let (command, rest) = words.split_first().map_or(("", &[][..]), |(c, r)| (*c, r));
        let positions = |from: usize| -> Result<Vec<usize>> {
            rest.get(from..)
                .unwrap_or(&[])
                .iter()
                .map(|w| {
                    w.parse()
                        .with_context(|| format!("{w:?} is not a position"))
                })
                .collect()
        };
        let goal = |i: usize| -> Result<InfId> {
            let word = rest.get(i).context("which goal? see `goals`")?;
            let id: u32 = word
                .parse()
                .with_context(|| format!("{word:?} is not a goal"))?;
            Ok(InfId::new(id))
        };
        let position = |i: usize| -> Result<usize> {
            let word = rest.get(i).context("which formula? see `goals`")?;
            word.parse()
                .with_context(|| format!("{word:?} is not a position"))
        };
        Ok(match command {
            "" => String::new(),
            "help" => HELP.to_owned(),
            "goals" => self.goals(),
            "rules" => {
                let rules = self.state.rules(goal(0)?, position(1)?)?;
                if rules.is_empty() {
                    "no rule acts on it".to_owned()
                } else {
                    rules
                        .iter()
                        .map(|r| match r.classical() {
                            Rule::Tensor | Rule::Mix => format!("{r} (with a split)"),
                            _ => r.to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join("  ")
                }
            }
            "apply" => {
                let (goal, position) = (goal(0)?, position(1)?);
                let word = rest.get(2).context("which rule? see `rules`")?;
                let rule: Rule = word.parse()?;
                let opened = self.state.apply(goal, position, rule, &positions(3)?)?;
                self.opened(&opened)
            }
            "undo" => match self.state.undo() {
                Some(goal) => format!("reopened {}", self.goal_line(goal)),
                None => "nothing to undo".to_owned(),
            },
            "close" => {
                let goals = match rest.first() {
                    Some(_) => vec![goal(0)?],
                    None => self.state.goals().collect(),
                };
                let mut text = String::new();
                for goal in goals {
                    let outcome = self.close(goal)?;
                    let _ = writeln!(text, "goal {}: {}", goal.get(), verdict(&outcome));
                }
                text.pop();
                if self.state.is_complete() {
                    text.push_str("\nno goal is open: `proof` checks the proof");
                }
                text
            }
            "show" => match rest.first() {
                None => self.state.derivation().to_string(),
                Some(&"latex") => latex::derivation(&self.state.derivation(), Form::Fragment),
                Some(&"typst") => typst::derivation(&self.state.derivation(), Form::Fragment),
                Some(&"svg") => svg::derivation(&self.state.derivation(), &Style::default()),
                Some(other) => bail!("show {other}? the formats are latex, typst and svg"),
            },
            "proof" => {
                let proof = self.state.proof()?;
                let mode = self.state.mode();
                match rest.first() {
                    Some(path) => {
                        io::write(Some(Path::new(path)), &serde_json::to_string(&proof)?)?;
                        format!("valid proof written to {path}")
                    }
                    None => {
                        let show = Show::text(self.view);
                        let stopped = || "stopped".to_owned();
                        match derivation(&proof, mode, &show, || false, stopped)? {
                            Shown::Written(tree) => format!("valid proof ({mode})\n{tree}"),
                            Shown::LeftOut(line) => format!("valid proof ({mode})\n{line}"),
                            Shown::Nothing => format!("valid proof ({mode})"),
                        }
                    }
                }
            }
            "save" => {
                let path = rest.first().context("save where? give a file")?;
                io::write(Some(Path::new(path)), &serde_json::to_string(&self.state)?)?;
                format!("session written to {path}")
            }
            "load" => {
                let path = rest.first().context("load what? give a file")?;
                self.state = load(Path::new(path))?;
                self.goals()
            }
            _ => bail!("unknown command {command:?}; `help` lists them"),
        })
    }

    /// Runs the search on a goal, stopped by the time limit or Ctrl-C.
    fn close(&mut self, goal: InfId) -> Result<Outcome> {
        clear_interrupt();
        let deadline = Deadline::start(self.timeout, Instant::now())?;
        let stop = || interrupted() || deadline.passed();
        match self.state.close(goal, &self.options, &self.view, stop) {
            Ok(outcome) => Ok(outcome),
            Err(Error::View(ViewError::TooLarge { size, limit })) => bail!(
                "the search proved the goal, but the derivation to graft is too large: its {} \
                 inferences with {} characters of sequents are estimated at {}, over the \
                 limit of {}; the goal stays open (--derivation-limit raises the limit)",
                count_text(size.inferences),
                count_text(size.characters),
                bytes_text(size.bytes()),
                bytes_text(limit)
            ),
            Err(Error::View(ViewError::Stopped)) => bail!(
                "the search proved the goal, but its derivation was not grafted before the \
                 time limit or the interrupt; the goal stays open"
            ),
            Err(error) => Err(error.into()),
        }
    }

    /// Lists the open goals, or says that none is.
    fn goals(&self) -> String {
        let goals: Vec<InfId> = self.state.goals().collect();
        if goals.is_empty() {
            return "no goal is open: `proof` checks the proof".to_owned();
        }
        goals
            .iter()
            .map(|&g| self.goal_line(g))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Lists goals just opened, or says that the step closed its goal.
    fn opened(&self, goals: &[InfId]) -> String {
        if goals.is_empty() {
            return if self.state.is_complete() {
                "closed; no goal is open: `proof` checks the proof".to_owned()
            } else {
                "closed".to_owned()
            };
        }
        goals
            .iter()
            .map(|&g| format!("opened {}", self.goal_line(g)))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Returns `goal G: ` and the goal's sequent with the position of every
    /// formula, two-sided in intuitionistic mode.
    fn goal_line(&self, goal: InfId) -> String {
        let sequent = self.state.goal(goal).unwrap_or(&[]);
        let forest = self.state.forest();
        let reading = self.state.reading();
        let mut line = format!("goal {}:", goal.get());
        let formula = |o| match &reading {
            Some(reading) => reading.formula(o).to_string(),
            None => forest.formula(o).to_string(),
        };
        let side = |o, side| {
            reading
                .as_ref()
                .is_none_or(|r: &Reading| r.position(o) == side)
        };
        for (i, &o) in sequent.iter().enumerate() {
            if side(o, Position::Input) && reading.is_some() {
                let _ = write!(
                    line,
                    "{}{i}: {}",
                    if i == 0 { " " } else { ", " },
                    formula(o)
                );
            }
        }
        line.push_str(" ⊢");
        let mut first = true;
        for (i, &o) in sequent.iter().enumerate() {
            if reading.is_none() || side(o, Position::Output) {
                let _ = write!(
                    line,
                    "{}{i}: {}",
                    if first { " " } else { ", " },
                    formula(o)
                );
                first = false;
            }
        }
        line
    }
}

/// Returns the verdict of a `close` as one line.
fn verdict(outcome: &Outcome) -> String {
    let context = format!(
        "{}, {}, {} engine",
        outcome.fragment.name_in(outcome.mode),
        outcome.mode,
        outcome.engine
    );
    match &outcome.verdict {
        Verdict::Proved(_) => format!("proved ({context})"),
        Verdict::Unprovable => format!("unprovable ({context}): the search was exhaustive"),
        Verdict::Unknown(reason) => {
            let why = match reason {
                Reason::Stopped => {
                    "the time limit was reached or the search was interrupted".to_owned()
                }
                Reason::RecursionLimit => format!("{reason}; raise it with --recursion-limit"),
                Reason::CopyBound(_) => format!("{reason}; raise it with --copies"),
                _ => reason.to_string(),
            };
            format!("unknown ({context}): {why}")
        }
    }
}
