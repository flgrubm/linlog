// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::derivation::{Derivation, InfId, Inference, Rule};
use crate::occurrences::{Forest, OccId, Position, Reading};
use std::fmt::{Display, Formatter, Result as FmtResult, Write};

/// A rendered subtree: lines of one width, the last of which is the
/// conclusion, whose text spans the columns `left..right`.
struct Block {
    /// The lines, top to bottom, each padded to `width` characters.
    lines: Vec<String>,
    /// The width in characters.
    width: usize,
    /// The column where the conclusion starts.
    left: usize,
    /// The column after the conclusion ends.
    right: usize,
}

/// The columns between two premises.
const GAP: usize = 3;

/// Returns the text of a sequent: `⊢` and its formulas, comma-separated;
/// two-sided under a reading, the hypotheses before `⊢` and the goal after
/// it.
pub(crate) fn sequent_text(
    forest: &Forest,
    reading: Option<&Reading>,
    sequent: &[OccId],
) -> String {
    let mut text = String::new();
    let Some(reading) = reading else {
        text.push('⊢');
        for (i, &o) in sequent.iter().enumerate() {
            write!(
                text,
                "{}{}",
                if i == 0 { " " } else { ", " },
                forest.formula(o)
            )
            .unwrap();
        }
        return text;
    };
    let mut goal = None;
    for &o in sequent {
        if reading.position(o) == Position::Output {
            goal = Some(o);
            continue;
        }
        write!(
            text,
            "{}{}",
            if text.is_empty() { "" } else { ", " },
            reading.formula(o)
        )
        .unwrap();
    }
    if !text.is_empty() {
        text.push(' ');
    }
    text.push('⊢');
    if let Some(goal) = goal {
        write!(text, " {}", reading.formula(goal)).unwrap();
    }
    text
}

/// Returns `s` padded with spaces to `width` characters.
fn pad(s: &str, width: usize) -> String {
    format!("{s:<width$}")
}

impl Derivation<'_> {
    /// Renders the subtree at `id`: the premises side by side, bottom-aligned,
    /// a bar spanning their conclusions with the rule's name after it, and
    /// the conclusion centred under the bar. An open goal is its sequent
    /// alone, with no bar above it.
    fn block(&self, id: InfId) -> Block {
        let Inference {
            sequent,
            rule,
            premises,
            ..
        } = self.inference(id);
        let conclusion = sequent_text(self.forest(), self.reading(), sequent);
        let width_of = |s: &str| s.chars().count();
        if *rule == Rule::Open {
            let width = width_of(&conclusion);
            return Block {
                lines: vec![conclusion],
                width,
                left: 0,
                right: width,
            };
        }

        // The premises in a row.
        let blocks: Vec<Block> = premises.iter().map(|&p| self.block(p)).collect();
        let height = blocks.iter().map(|b| b.lines.len()).max().unwrap_or(0);
        let mut lines = vec![String::new(); height];
        let (mut span_left, mut span_right) = (0, 0);
        for (i, b) in blocks.iter().enumerate() {
            let offset = width_of(&lines[0]) + if i == 0 { 0 } else { GAP };
            if i == 0 {
                span_left = b.left;
            }
            span_right = offset + b.right;
            for (row, line) in lines.iter_mut().enumerate() {
                let above = height - b.lines.len();
                let text = if row < above {
                    ""
                } else {
                    &b.lines[row - above]
                };
                *line = pad(line, offset) + &pad(text, b.width);
            }
        }

        // The bar covers the premises' conclusions and the conclusion,
        // whichever is wider, centred on the other.
        let c_width = width_of(&conclusion);
        let span = span_right - span_left;
        let bar_width = c_width.max(span);
        let mut bar_left = span_left as isize - ((bar_width - span) / 2) as isize;
        if bar_left < 0 {
            let shift = (-bar_left) as usize;
            for line in &mut lines {
                *line = pad("", shift) + line;
            }
            bar_left = 0;
        }
        let bar_left = bar_left as usize;
        let c_left = bar_left + (bar_width - c_width) / 2;
        lines.push(format!(
            "{}{} {rule}",
            pad("", bar_left),
            "─".repeat(bar_width)
        ));
        lines.push(pad("", c_left) + &conclusion);
        let width = lines.iter().map(|l| width_of(l)).max().unwrap();
        for line in &mut lines {
            *line = pad(line, width);
        }
        Block {
            lines,
            width,
            left: c_left,
            right: c_left + c_width,
        }
    }
}

impl Display for Derivation<'_> {
    /// Draws the derivation as a tree of sequents, one line per row,
    /// without a trailing newline.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let block = self.block(self.root());
        for (i, line) in block.lines.iter().enumerate() {
            if i > 0 {
                f.write_str("\n")?;
            }
            f.write_str(line.trim_end())?;
        }
        Ok(())
    }
}
