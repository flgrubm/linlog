// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The text rendering of a derivation: a tree of sequents with a bar and
//! the rule's name over each conclusion. The layout is computed in two
//! passes over the inferences, sizes from the leaves down and positions
//! from the root up, and written row by row, so that drawing a tree
//! takes time proportional to its text and memory proportional to its
//! inferences.

use super::derivation::{Derivation, InfId, Rule};
use crate::occurrences::{Forest, OccId, Position, Reading};
use std::fmt::{Display, Formatter, Result as FmtResult, Write};

/// The columns between two premises.
const GAP: usize = 3;

/// Where an inference and the subtree above it go, within the box that
/// holds the subtree.
#[derive(Clone, Copy, Debug, Default)]
struct Place {
    /// The left end of the box: relative to the box of the inference
    /// below after the first pass, in the whole drawing after the second.
    x: usize,
    /// The width of the box in characters.
    width: usize,
    /// The lines of the box.
    height: usize,
    /// The inferences below it, down to the root.
    depth: usize,
    /// The column of the box where the conclusion starts.
    left: usize,
    /// The characters of the conclusion.
    conclusion: usize,
    /// The column of the box where the bar starts and the bar's length, or
    /// none for an open goal.
    bar: Option<(usize, usize)>,
}

/// A piece of one row of the drawing: an inference's conclusion or its
/// bar with the rule's name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Piece {
    /// The row, from the top.
    row: usize,
    /// The column it starts in.
    column: usize,
    /// The inference.
    id: InfId,
    /// Whether it is the bar rather than the conclusion.
    bar: bool,
}

/// A writer that counts the characters written to it.
struct Count(usize);

impl Write for Count {
    fn write_str(&mut self, s: &str) -> FmtResult {
        self.0 += s.chars().count();
        Ok(())
    }
}

/// Writes `width` spaces.
fn spaces(out: &mut impl Write, width: usize) -> FmtResult {
    write!(out, "{:width$}", "")
}

/// Writes a sequent: `⊢` and its formulas, comma-separated; two-sided
/// under a reading, the hypotheses before `⊢` and the goal after it.
fn write_sequent(
    out: &mut impl Write,
    forest: &Forest,
    reading: Option<&Reading>,
    sequent: &[OccId],
) -> FmtResult {
    let Some(reading) = reading else {
        out.write_char('⊢')?;
        for (i, &o) in sequent.iter().enumerate() {
            out.write_str(if i == 0 { " " } else { ", " })?;
            write!(out, "{}", forest.formula(o))?;
        }
        return Ok(());
    };
    let (mut goal, mut first) = (None, true);
    for &o in sequent {
        if reading.position(o) == Position::Output {
            goal = Some(o);
            continue;
        }
        if !first {
            out.write_str(", ")?;
        }
        first = false;
        write!(out, "{}", reading.formula(o))?;
    }
    if !first {
        out.write_char(' ')?;
    }
    out.write_char('⊢')?;
    if let Some(goal) = goal {
        write!(out, " {}", reading.formula(goal))?;
    }
    Ok(())
}

/// Returns the text of a sequent: `⊢` and its formulas, comma-separated;
/// two-sided under a reading, the hypotheses before `⊢` and the goal after
/// it.
pub(crate) fn sequent_text(
    forest: &Forest,
    reading: Option<&Reading>,
    sequent: &[OccId],
) -> String {
    let mut text = String::new();
    write_sequent(&mut text, forest, reading, sequent).unwrap();
    text
}

impl Derivation<'_> {
    /// Lays the tree out: the premises of an inference side by side,
    /// bottom-aligned and [`GAP`] columns apart, a bar spanning their
    /// conclusions or the conclusion, whichever is wider, with the rule's
    /// name after it, and the conclusion centred under the bar. An open
    /// goal is its sequent alone, with no bar above it. Returns every
    /// inference's place, its box positioned in the whole drawing.
    fn layout(&self) -> Vec<Place> {
        let mut places = vec![Place::default(); self.inferences().len()];
        // Sizes: an inference comes after its premises.
        for (i, inference) in self.inferences().iter().enumerate() {
            let mut count = Count(0);
            write_sequent(
                &mut count,
                self.forest(),
                self.reading(),
                &inference.sequent,
            )
            .expect("counting never fails");
            let conclusion = count.0;
            if inference.rule == Rule::Open {
                places[i] = Place {
                    width: conclusion,
                    height: 1,
                    conclusion,
                    ..Place::default()
                };
                continue;
            }
            // The premises in a row, and the span of their conclusions.
            let (mut row, mut height) = (0, 0);
            let (mut span_left, mut span_right) = (0, 0);
            for (k, p) in inference.premises.iter().enumerate() {
                let place = &mut places[p.index()];
                place.x = row + if k == 0 { 0 } else { GAP };
                if k == 0 {
                    span_left = place.left;
                }
                span_right = place.x + place.left + place.conclusion;
                row = place.x + place.width;
                height = height.max(place.height);
            }
            // The bar covers the premises' conclusions and the conclusion,
            // whichever is wider, centred on the other; a bar that would
            // start left of the box moves the premises right instead.
            let span = span_right - span_left;
            let bar_width = conclusion.max(span);
            let overhang = (bar_width - span) / 2;
            let shift = overhang.saturating_sub(span_left);
            for p in &inference.premises {
                places[p.index()].x += shift;
            }
            let bar_left = span_left + shift - overhang;
            let left = bar_left + (bar_width - conclusion) / 2;
            let name = inference.rule.name().chars().count();
            let premises = if inference.premises.is_empty() {
                0
            } else {
                row + shift
            };
            places[i] = Place {
                x: 0,
                width: premises
                    .max(bar_left + bar_width + 1 + name)
                    .max(left + conclusion),
                height: height + 2,
                depth: 0,
                left,
                conclusion,
                bar: Some((bar_left, bar_width)),
            };
        }
        // Positions: an inference comes before its premises.
        for (i, inference) in self.inferences().iter().enumerate().rev() {
            let (x, depth) = (places[i].x, places[i].depth);
            for p in &inference.premises {
                places[p.index()].x += x;
                places[p.index()].depth = depth + 1;
            }
        }
        places
    }

    /// Returns the width in characters and the number of lines of the tree
    /// that [`Display`] draws, without drawing it.
    pub fn text_size(&self) -> (usize, usize) {
        let root = self.layout()[self.root().index()];
        (root.width, root.height)
    }
}

impl Display for Derivation<'_> {
    /// Draws the derivation as a tree of sequents, one line per row,
    /// without a trailing newline.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let places = self.layout();
        let height = places[self.root().index()].height;
        // The root's conclusion is the last row, its bar the one above,
        // and every inference further up is two rows higher.
        let mut pieces = Vec::with_capacity(2 * places.len());
        for (i, place) in places.iter().enumerate() {
            let id = InfId::new(i as u32);
            let row = height - 1 - 2 * place.depth;
            pieces.push(Piece {
                row,
                column: place.x + place.left,
                id,
                bar: false,
            });
            if let Some((left, _)) = place.bar {
                pieces.push(Piece {
                    row: row - 1,
                    column: place.x + left,
                    id,
                    bar: true,
                });
            }
        }
        pieces.sort_unstable();
        let (mut row, mut column) = (0, 0);
        for piece in pieces {
            while row < piece.row {
                f.write_char('\n')?;
                (row, column) = (row + 1, 0);
            }
            spaces(f, piece.column - column)?;
            let (place, inference) = (&places[piece.id.index()], self.inference(piece.id));
            column = piece.column;
            match place.bar {
                Some((_, width)) if piece.bar => {
                    for _ in 0..width {
                        f.write_char('─')?;
                    }
                    let name = inference.rule.name();
                    write!(f, " {name}")?;
                    column += width + 1 + name.chars().count();
                }
                _ => {
                    write_sequent(f, self.forest(), self.reading(), &inference.sequent)?;
                    column += place.conclusion;
                }
            }
        }
        Ok(())
    }
}
