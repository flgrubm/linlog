// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proof structures as formula trees under their axiom links: the
//! literals side by side along the top in occurrence order, which is left
//! to right in every tree, each connective one layer below the lower of
//! its premises and halfway between them, the conclusions on the bottom
//! layer with an edge hanging from each, and every link a half-ellipse
//! over the two literals it joins, as high as it is wide times a fixed
//! ratio. Two links over nested pairs of literals are nested half-ellipses
//! of one shape, which never cross; links over interleaved pairs cross.

use super::font::{AXIS, HEIGHT};
use super::{NOTATION, PLAIN, Style, document, escaped, run, text};
use crate::nets::{NetError, ProofStructure};
use crate::occurrences::OccId;
use crate::sequents::Kind;
use std::fmt::Write;

/// How far below a literal's baseline its premise edge ends.
const FOOT: i64 = 100;

/// The height of the middle of `⅋` above the baseline: unlike `⊗`, it
/// stands on the baseline instead of being centred on the math axis.
const PAR_MIDDLE: i64 = 344;

/// Returns the point `by` along the segment from `from` towards `to`.
fn towards(from: (i64, i64), to: (i64, i64), by: i64) -> (i64, i64) {
    let (dx, dy) = ((to.0 - from.0) as f64, (to.1 - from.1) as f64);
    let scale = by as f64 / dx.hypot(dy);
    (
        from.0 + (dx * scale).round() as i64,
        from.1 + (dy * scale).round() as i64,
    )
}

/// Returns a proof structure as an SVG document, titled with its sequent
/// in plain text.
pub(super) fn draw(net: &ProofStructure, style: &Style) -> String {
    let forest = net.forest();
    let n = forest.len();
    let line_height = i64::from(style.line_height);
    let (gap, radius) = (i64::from(style.literal_gap), i64::from(style.node_radius));
    let (margin, stroke) = (i64::from(style.margin), i64::from(style.stroke_width));
    let label_size = i64::from(style.label_size);

    // The literals from left to right, and every connective halfway
    // between its premises, one layer below the lower one.
    let mut x = vec![0; n];
    let mut layer = vec![0; n];
    let mut literals = Vec::new();
    let (mut next, mut formula) = (margin, String::new());
    for o in forest.ids().filter(|&o| forest.is_literal(o)) {
        formula.clear();
        NOTATION.term(&mut formula, forest.sequent(), forest.term(o), false);
        let label = run(&formula, 1000);
        x[o.index()] = next + label.width / 2;
        next += label.width + gap;
        literals.push((o, label));
    }
    for o in forest.ids().rev() {
        if let (Some(l), Some(r)) = (forest.left(o), forest.right(o)) {
            layer[o.index()] = 1 + layer[l.index()].max(layer[r.index()]);
            x[o.index()] = (x[l.index()] + x[r.index()]) / 2;
        }
    }
    let bottom = forest.roots().iter().map(|r| layer[r.index()]).max();
    let bottom = bottom.unwrap_or(0);
    for &r in forest.roots() {
        if !forest.is_literal(r) {
            layer[r.index()] = bottom;
        }
    }

    // The links' arcs above the literals decide where the literals'
    // baseline is.
    let ratio = i64::from(style.link_height);
    let arc = |(p, q): (OccId, OccId)| {
        let (a, b) = (
            x[p.index()].min(x[q.index()]),
            x[p.index()].max(x[q.index()]),
        );
        let rx = (b - a) / 2;
        (a, b, rx, rx * ratio / 1000)
    };
    let highest = net.links().iter().map(|&l| arc(l).3).max();
    let baseline = margin + stroke + highest.unwrap_or(0) + HEIGHT;
    let y = |o: OccId| match forest.is_literal(o) {
        true => baseline - AXIS,
        false => baseline - AXIS + layer[o.index()] * line_height,
    };
    let end = baseline - AXIS + bottom * line_height + line_height / 2;

    // A switching cycle, as its edges, when the structure has one.
    let mut cycle = Vec::new();
    if let Err(NetError::SwitchingCycle(vertices)) = net.is_correct() {
        let next = vertices.iter().cycle().skip(1);
        cycle = vertices
            .iter()
            .zip(next)
            .map(|(&a, &b)| (a.min(b), a.max(b)))
            .collect();
        cycle.sort_unstable();
    }
    let in_cycle = |a: OccId, b: OccId| cycle.binary_search(&(a.min(b), a.max(b))).is_ok();

    let (mut lines, mut pars, mut links) = (String::new(), String::new(), String::new());
    let (mut marked, mut texts) = (String::new(), String::new());
    let dash = format!("{} {}", 3 * stroke, 3 * stroke);
    for (o, label) in &literals {
        let id = format!(r#" id="o{}""#, o.get());
        text(
            &mut texts,
            x[o.index()] - label.width / 2,
            baseline,
            label,
            &id,
        );
    }
    for o in forest.ids().filter(|&o| !forest.is_literal(o)) {
        let centre = (x[o.index()], y(o));
        writeln!(
            lines,
            r#"<circle id="o{}" cx="{}" cy="{}" r="{radius}"/>"#,
            o.get(),
            centre.0,
            centre.1
        )
        .unwrap();
        let (symbol, middle) = match forest.kind(o) {
            Kind::Par => ("⅋", PAR_MIDDLE),
            _ => ("⊗", AXIS),
        };
        let label = run(symbol, label_size);
        let symbol_y = centre.1 + middle * label_size / 1000;
        let size = format!(r#" font-size="{label_size}""#);
        text(
            &mut texts,
            centre.0 - label.width / 2,
            symbol_y,
            &label,
            &size,
        );
        for child in forest.children(o) {
            let target = (x[child.index()], y(child));
            let from = towards(centre, target, radius);
            let to = match forest.is_literal(child) {
                true => (target.0, baseline + FOOT),
                false => towards(target, centre, radius),
            };
            let par = forest.kind(o) == Kind::Par;
            let out = match (in_cycle(o, child), par) {
                (true, _) => &mut marked,
                (false, true) => &mut pars,
                (false, false) => &mut lines,
            };
            write!(out, r#"<path d="M{} {}L{} {}""#, from.0, from.1, to.0, to.1).unwrap();
            if par && in_cycle(o, child) {
                write!(out, r#" stroke-dasharray="{dash}""#).unwrap();
            }
            out.push_str("/>\n");
        }
    }
    for &r in forest.roots() {
        let start = match forest.is_literal(r) {
            true => baseline + FOOT,
            false => y(r) + radius,
        };
        let column = x[r.index()];
        writeln!(lines, r#"<path d="M{column} {start}V{end}"/>"#).unwrap();
    }
    let top = baseline - HEIGHT;
    for &(p, q) in net.links() {
        let (a, b, rx, ry) = arc((p, q));
        let out = if in_cycle(p, q) {
            &mut marked
        } else {
            &mut links
        };
        writeln!(
            out,
            r#"<path id="l{}-{}" d="M{a} {top}A{rx} {ry} 0 0 1 {b} {top}"/>"#,
            p.min(q).get(),
            p.max(q).get()
        )
        .unwrap();
    }

    let group = |colour: &str, extra: &str, content: &str| {
        format!(
            "<g fill=\"none\" stroke=\"{}\" stroke-width=\"{stroke}\"{extra}>\n{content}</g>\n",
            escaped(colour)
        )
    };
    let mut body = group(&style.line, "", &lines);
    body += &group(&style.par, &format!(r#" stroke-dasharray="{dash}""#), &pars);
    body += &group(&style.link, "", &links);
    if !marked.is_empty() {
        body += &group(&style.highlight, "", &marked);
    }
    body += &texts;
    let mut title = String::new();
    PLAIN.one_sided(&mut title, forest.sequent());
    let width = if literals.is_empty() {
        2 * margin
    } else {
        next - gap + margin
    };
    document(style, &title, width, end + stroke + margin, &body)
}
