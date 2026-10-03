// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Markdown tables from the rows `run` wrote. A configuration is a CSV
//! file's name with the mode, engine, threads, portfolio and test period
//! of its rows, since every file of a baseline is one run of `run` with
//! its own options. A problem counts once per configuration: its first run
//! gives the verdict, the median over its runs the time. Rows of a forced
//! engine that does not apply (`refused`) are left out.

use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::path::PathBuf;

/// One CSV row, as a map from column to value.
pub(crate) struct Row(HashMap<String, String>);

impl Row {
    /// The value of a column, empty when the column is absent.
    pub(crate) fn get(&self, column: &str) -> &str {
        self.0.get(column).map_or("", String::as_str)
    }
}

/// The runs of one problem in one configuration.
pub(crate) struct Runs<'a> {
    /// The first run.
    pub(crate) first: &'a Row,
    /// The time of every run, in milliseconds.
    pub(crate) times: Vec<f64>,
    /// Whether a sequential run waited for a CPU for more than a hundredth
    /// of its time and a millisecond: another process slowed it down.
    pub(crate) disturbed: bool,
}

impl Runs<'_> {
    /// The median of the times.
    pub(crate) fn median(&self) -> f64 {
        let mut times = self.times.clone();
        times.sort_by(f64::total_cmp);
        times[times.len() / 2]
    }

    /// Whether the verdict is decided.
    pub(crate) fn solved(&self) -> bool {
        matches!(self.first.get("verdict"), "proved" | "unprovable")
    }

    /// Whether the verdict contradicts the expected one, or a proof failed
    /// the checker: a bug for a generated problem, a bug or a wrong header
    /// for an LLTP one.
    fn wrong(&self) -> bool {
        let (verdict, expected) = (self.first.get("verdict"), self.first.get("expected"));
        (verdict == "proved" && (expected == "unprovable" || self.first.get("checked") != "ok"))
            || (verdict == "unprovable" && expected == "provable")
    }

    /// The cell of a scaling table: the median time and a mark for the
    /// verdict.
    pub(crate) fn cell(&self) -> String {
        let row = self.first;
        let mark = match (row.get("verdict"), row.get("reason")) {
            ("proved", _) => "✓",
            ("unprovable", _) => "✗",
            (_, "timeout" | "killed") => return format!("> {}", seconds(row.get("timeout_s"))),
            (_, "copy_bound") => "? bound",
            (_, "context_too_wide") => "? wide",
            (_, "recursion_limit") => "? depth",
            (verdict, _) => verdict,
        };
        let wrong = if self.wrong() { " MISMATCH" } else { "" };
        let disturbed = if self.disturbed { " †" } else { "" };
        format!("{} {mark}{wrong}{disturbed}", time(self.median()))
    }
}

/// The runs of one configuration on one family.
struct Group<'a> {
    /// The family.
    family: String,
    /// The configuration's label.
    config: String,
    /// The runs of every problem, in the order first seen.
    problems: Vec<(String, Runs<'a>)>,
}

/// Prints the tables for the CSV files.
pub fn summary(files: &[PathBuf]) -> Result<()> {
    let rows = read(files)?;

    // Runs grouped by configuration and problem, in the order first seen.
    let config = |row: &Row| {
        let mut label = format!(
            "{}: {} {} j{}",
            row.get("file"),
            row.get("mode"),
            row.get("engine_requested"),
            row.get("jobs")
        );
        if row.get("portfolio") == "true" {
            label.push_str(" portfolio");
        }
        if !row.get("test_period").is_empty() {
            label.push_str(&format!(" period {}", row.get("test_period")));
        }
        if !matches!(row.get("bias"), "" | "auto") {
            label.push_str(&format!(" bias {}", row.get("bias")));
        }
        let forward = row.get("forward_copies");
        if !forward.is_empty()
            && forward != linlog::search::Options::DEFAULT_FORWARD_COPIES.to_string()
        {
            label.push_str(&format!(" forward {forward}"));
        }
        // A file from before the bound has no such column, and ran
        // without one.
        let memory = row.get("memory_limit");
        if memory == "0" {
            label.push_str(" memory none");
        } else if !memory.is_empty()
            && memory != linlog::search::Options::DEFAULT_MEMORY_LIMIT.to_string()
        {
            label.push_str(&format!(" memory {memory}"));
        }
        // A file from before the column ran every pool from its start.
        if !row.get("pool_after").is_empty() {
            label.push_str(&format!(" pool after {}s", row.get("pool_after")));
        }
        label
    };
    let mut groups: Vec<Group> = Vec::new();
    for row in &rows {
        let (family, label, problem) = (row.get("family"), config(row), row.get("problem"));
        let at = groups
            .iter()
            .position(|g| g.family == family && g.config == label);
        let at = at.unwrap_or_else(|| {
            groups.push(Group {
                family: family.to_owned(),
                config: label,
                problems: Vec::new(),
            });
            groups.len() - 1
        });
        let problems = &mut groups[at].problems;
        let time: f64 = row.get("time_ms").parse().unwrap_or(0.0);
        let wait: f64 = row.get("wait_ms").parse().unwrap_or(0.0);
        let disturbed = row.get("jobs") == "1" && wait > (time / 100.0).max(1.0);
        match problems.iter_mut().find(|(p, _)| p == problem) {
            Some((_, runs)) => {
                runs.times.push(time);
                runs.disturbed |= disturbed;
            }
            None => problems.push((
                problem.to_owned(),
                Runs {
                    first: row,
                    times: vec![time],
                    disturbed,
                },
            )),
        }
    }

    let disturbed = groups
        .iter()
        .flat_map(|g| &g.problems)
        .filter(|(_, r)| r.disturbed)
        .count();
    println!("## Solved within the time limit\n");
    println!(
        "Sequential problems that waited for a CPU for over 1 % of their time \
         (another process slowed them down; `†` in the tables below): {disturbed}.\n"
    );
    println!(
        "| family | configuration | engines | problems | solved | proved | refuted | timeout | bound | other | mismatch | median solved | total solved |"
    );
    println!("|---|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|");
    for Group {
        family,
        config: label,
        problems,
    } in &groups
    {
        let count = |test: &dyn Fn(&Runs) -> bool| problems.iter().filter(|(_, r)| test(r)).count();
        let verdict = |v: &'static str| move |r: &Runs| r.first.get("verdict") == v;
        let reason = |v: &'static str| move |r: &Runs| r.first.get("reason") == v;
        let timeouts = count(&reason("timeout")) + count(&reason("killed"));
        let bound = count(&reason("copy_bound"));
        let solved: Vec<f64> = problems
            .iter()
            .filter(|(_, r)| r.solved())
            .map(|(_, r)| r.median())
            .collect();
        let mut engines: Vec<&str> = problems
            .iter()
            .map(|(_, r)| r.first.get("engine"))
            .collect();
        engines.sort_unstable();
        engines.dedup();
        engines.retain(|e| !e.is_empty());
        let mut sorted = solved.clone();
        sorted.sort_by(f64::total_cmp);
        println!(
            "| {family} | {label} | {} | {} | {} | {} | {} | {timeouts} | {bound} | {} | {} | {} | {} |",
            engines.join(", "),
            problems.len(),
            solved.len(),
            count(&verdict("proved")),
            count(&verdict("unprovable")),
            problems.len() - solved.len() - timeouts - bound,
            count(&|r: &Runs| r.wrong()),
            sorted
                .get(sorted.len() / 2)
                .map_or("–".to_owned(), |&t| time(t)),
            if solved.is_empty() {
                "–".to_owned()
            } else {
                time(solved.iter().sum())
            },
        );
    }

    // One table per family of generated or listed problems: a row per
    // problem, a column per configuration.
    let mut families: Vec<&str> = Vec::new();
    for row in rows.iter().filter(|r| r.get("source") != "lltp") {
        if !families.contains(&row.get("family")) {
            families.push(row.get("family"));
        }
    }
    for family in families {
        let columns: Vec<&Group> = groups.iter().filter(|g| g.family == family).collect();
        let mut problems: Vec<&str> = Vec::new();
        for group in &columns {
            for (problem, _) in &group.problems {
                if !problems.contains(&problem.as_str()) {
                    problems.push(problem);
                }
            }
        }
        println!("\n## {family}\n");
        let labels: Vec<&str> = columns.iter().map(|g| g.config.as_str()).collect();
        println!("| problem | {} |", labels.join(" | "));
        println!("|---|{}", "--:|".repeat(labels.len()));
        for problem in problems {
            let cells: Vec<String> = columns
                .iter()
                .map(|group| {
                    group
                        .problems
                        .iter()
                        .find(|(p, _)| p == problem)
                        .map_or("".to_owned(), |(_, r)| r.cell())
                })
                .collect();
            println!("| {problem} | {} |", cells.join(" | "));
        }
    }
    Ok(())
}

/// The rows of the CSV files, each with its file's name in the column
/// `file`, without those of a forced engine that does not apply.
pub(crate) fn read(files: &[PathBuf]) -> Result<Vec<Row>> {
    let mut rows = Vec::new();
    for file in files {
        let text =
            std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
        let mut lines = text.lines();
        let Some(header) = lines.next() else { continue };
        let columns: Vec<&str> = header.split(',').collect();
        let stem = file.file_stem().unwrap_or_default().to_string_lossy();
        for line in lines.filter(|l| !l.is_empty() && !l.starts_with("source,")) {
            let values: Vec<&str> = line.split(',').collect();
            if values.len() != columns.len() {
                bail!(
                    "{}: a row of {} fields: {line}",
                    file.display(),
                    values.len()
                );
            }
            let row = columns
                .iter()
                .zip(values)
                .map(|(c, v)| ((*c).to_owned(), v.to_owned()));
            let mut row: HashMap<String, String> = row.collect();
            row.insert("file".to_owned(), stem.to_string());
            rows.push(Row(row));
        }
    }
    rows.retain(|row| row.get("verdict") != "refused");
    Ok(rows)
}

/// Formats milliseconds for a table: microseconds below one millisecond,
/// milliseconds below a second, seconds above.
pub(crate) fn time(ms: f64) -> String {
    if ms < 1.0 {
        format!("{:.0} µs", ms * 1000.0)
    } else if ms < 1000.0 {
        format!("{ms:.1} ms")
    } else {
        format!("{:.2} s", ms / 1000.0)
    }
}

/// Formats a timeout in seconds, as the CSV holds it.
fn seconds(value: &str) -> String {
    format!("{} s", value.parse::<f64>().unwrap_or(0.0))
}
