// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Derivations as proof trees, laid out bottom-up by subtree width: the
//! premises of an inference side by side, their conclusions centred over
//! its conclusion, an inference line spanning both with the rule's name to
//! its right, and the root at the bottom. Every row is one line height
//! above the row below it.

use super::font::{AXIS, DEPTH, HEIGHT};
use super::{NOTATION, PLAIN, Run, Style, document, run, text};
use crate::export::notation::{Step, walk};
use crate::proofs::{Derivation, Rule};
use std::fmt::Write;

/// Where an inference and the subtree above it go, within the box that
/// holds the subtree and the rule names in it.
#[derive(Clone, Debug, Default)]
struct Place {
    /// The left end of the box, relative to the left end of the box of the
    /// inference below; 0 for the root.
    offset: i64,
    /// The width of the box.
    width: i64,
    /// The conclusion, laid out.
    conclusion: Run,
    /// The left end of the conclusion within the box.
    left: i64,
    /// The ends of the inference line within the box, or none for an
    /// open goal.
    bar: Option<(i64, i64)>,
    /// The rule's name, laid out; empty for an open goal.
    label: Run,
}

/// Returns a derivation as an SVG document of its proof tree, titled with
/// its conclusion in plain text.
pub(super) fn draw(derivation: &Derivation, style: &Style) -> String {
    let (forest, reading) = (derivation.forest(), derivation.reading());
    let line_height = i64::from(style.line_height);
    let label_size = i64::from(style.label_size);
    let (gap, label_gap) = (i64::from(style.premise_gap), i64::from(style.label_gap));
    let (margin, stroke) = (i64::from(style.margin), i64::from(style.stroke_width));
    // An inference line lies halfway between the lowest reach of the
    // premises' text and the highest reach of the conclusion's; a rule's
    // name has its math axis on the line; the dots over an open goal stand
    // where its premises would.
    let rise = (line_height + HEIGHT - DEPTH) / 2;
    let dots = run("⋮", 1000);
    let dots_rise = line_height - DEPTH;
    let dots_height = 560;

    // Up the tree: sizes, the premises' offsets, and the highest point of
    // the drawing relative to the root's baseline.
    let mut places = vec![Place::default(); derivation.inferences().len()];
    let mut top = -HEIGHT;
    let mut sequent = String::new();
    walk(derivation, |step| {
        let Step::Exit(id, depth) = step else {
            return;
        };
        let inference = derivation.inference(id);
        sequent.clear();
        NOTATION.sequent(&mut sequent, forest, reading, &inference.sequent, false);
        let conclusion = run(&sequent, 1000);
        let baseline = -(depth as i64) * line_height;
        top = top.min(baseline - HEIGHT);
        let width = conclusion.width;
        if inference.rule == Rule::Open {
            top = top.min(baseline - dots_rise - dots_height);
            places[id.index()] = Place {
                width: width.max(dots.width),
                left: (dots.width - width).max(0) / 2,
                conclusion,
                ..Place::default()
            };
            return;
        }
        let label = run(inference.rule.name(), label_size);
        top = top.min(baseline - rise - stroke / 2);
        top = top.min(baseline - rise + (AXIS - HEIGHT) * label_size / 1000);

        // The premises side by side from 0, and the span of their
        // conclusions.
        let (mut x, mut span) = (0, None);
        for &p in &inference.premises {
            let place = &mut places[p.index()];
            place.offset = x;
            let (start, end) = (x + place.left, x + place.left + place.conclusion.width);
            span = Some(span.map_or((start, end), |(s, _)| (s, end)));
            x += place.width + gap;
        }
        let row = (x - gap).max(0);
        let (left, bar) = match span {
            Some((start, end)) => {
                let left = (start + end) / 2 - width / 2;
                (left, (start.min(left), end.max(left + width)))
            }
            None => (0, (0, width)),
        };
        let shift = -left.min(0);
        for &p in &inference.premises {
            places[p.index()].offset += shift;
        }
        let right = row.max(bar.1 + label_gap + label.width);
        places[id.index()] = Place {
            offset: 0,
            width: right + shift,
            conclusion,
            left: left + shift,
            bar: Some((bar.0 + shift, bar.1 + shift)),
            label,
        };
    });

    // Down the tree: every box at its place, and what it draws.
    let mut xs = vec![0; places.len()];
    xs[derivation.root().index()] = margin;
    let (mut lines, mut texts) = (String::new(), String::new());
    walk(derivation, |step| {
        let Step::Enter(id, depth) = step else {
            return;
        };
        let (place, x) = (&places[id.index()], xs[id.index()]);
        for &p in &derivation.inference(id).premises {
            xs[p.index()] = x + places[p.index()].offset;
        }
        let y = margin - top - depth as i64 * line_height;
        let id = format!(r#" id="i{}""#, id.get());
        text(&mut texts, x + place.left, y, &place.conclusion, &id);
        match place.bar {
            None => {
                let dots_x = x + (place.width - dots.width) / 2;
                text(&mut texts, dots_x, y - dots_rise, &dots, "");
            }
            Some((start, end)) => {
                let y = y - rise;
                writeln!(lines, r#"<path d="M{} {y}H{}"/>"#, x + start, x + end).unwrap();
                let size = format!(r#" font-size="{label_size}""#);
                let baseline = y + AXIS * label_size / 1000;
                text(
                    &mut texts,
                    x + end + label_gap,
                    baseline,
                    &place.label,
                    &size,
                );
            }
        }
    });

    let root = derivation.root();
    let mut title = String::new();
    let conclusion = &derivation.inference(root).sequent;
    PLAIN.sequent(&mut title, forest, reading, conclusion, false);
    let body = format!(
        "<g fill=\"none\" stroke=\"{}\" stroke-width=\"{stroke}\">\n{lines}</g>\n{texts}",
        super::escaped(&style.line)
    );
    let width = places[root.index()].width + 2 * margin;
    document(style, &title, width, DEPTH - top + 2 * margin, &body)
}
