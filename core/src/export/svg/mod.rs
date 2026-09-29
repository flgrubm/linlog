// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! SVG: a sequent as one line of text, and a whole SVG document for each
//! drawing.
//!
//! The layout is computed here, not by the viewer: every width is the sum
//! of the advances of the Euler Math font, from a table committed with the
//! code, so the output is deterministic and needs no font at run time. The
//! document asks for `"Euler Math", "Neo Euler", serif` and does not embed
//! the font; every `<text>` carries its computed width as `textLength`, so
//! a viewer that sets it in another font still fits the layout. Letters of
//! atom names are the mathematical italic characters, which a math font
//! sets as math italic, and rule names stay upright; the `⊥` of a negated
//! atom is a superscript, the `₁` and `₂` of a rule name are subscripts.
//! All coordinates are integers in thousandths of an em of formula text
//! ([`Style::font_size`] pixels), so the text is selectable and the
//! drawing scales without loss.
//!
//! # Examples
//!
#![cfg_attr(feature = "parse", doc = "```")]
#![cfg_attr(not(feature = "parse"), doc = "```ignore")]
//! use linlog::Sequent;
//! use linlog::export::svg::{self, Style};
//!
//! let sequent: Sequent = "A |- A".parse()?;
//! let svg = svg::sequent(&sequent, &Style::default());
//! assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
//! assert!(svg.contains("<title>⊢ A⊥, A</title>"));
//! # Ok::<(), linlog::Error>(())
//! ```

/// The metrics of the Euler Math font.
mod font;

use super::notation::Notation;
use crate::occurrences::Reading;
use crate::sequents::Sequent;
use font::{DEPTH, HEIGHT, LOWER, RAISE, SCRIPT, advance};
use std::fmt::Write;

/// The sizes, distances and colours of a drawing. Lengths are in
/// thousandths of an em of formula text, whose size in pixels is
/// [`font_size`](Self::font_size); colours are CSS colours.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Style {
    /// The size of formula text in pixels, which scales the whole drawing.
    pub font_size: u32,
    /// The size of rule names and of the connectives in a proof net's
    /// nodes, in thousandths of the size of formula text.
    pub label_size: u32,
    /// The distance from the baseline of a conclusion to the baseline of
    /// its premises, and from one layer of a proof net to the next.
    pub line_height: u32,
    /// The space between two premises side by side.
    pub premise_gap: u32,
    /// The space between two literals of a proof net.
    pub literal_gap: u32,
    /// The space between an inference line and its rule name.
    pub label_gap: u32,
    /// The empty border around the drawing.
    pub margin: u32,
    /// The thickness of every line.
    pub stroke_width: u32,
    /// The height of an axiom link's arc, in thousandths of half its
    /// width, so that the arc over a pair of literals nested between two
    /// others stays below theirs.
    pub link_height: u32,
    /// The radius of the circle of a `⊗` or `⅋` node of a proof net.
    pub node_radius: u32,
    /// The colour of text.
    pub text: String,
    /// The colour of inference lines, of a proof net's nodes, of the
    /// premise edges of its `⊗` nodes and of its conclusions.
    pub line: String,
    /// The colour of the premise edges of a `⅋` node, which are also
    /// dashed: a switching keeps one of the two.
    pub par: String,
    /// The colour of axiom links.
    pub link: String,
    /// The colour of the edges of a switching cycle, when a proof
    /// structure has one.
    pub highlight: String,
    /// The colour the drawing is filled with, or none for a transparent
    /// background.
    pub background: Option<String>,
}

impl Default for Style {
    /// Returns black lines and text at 16 pixels to the em on a
    /// transparent background, with `⅋` edges blue and a switching cycle
    /// orange.
    fn default() -> Self {
        Self {
            font_size: 16,
            label_size: 800,
            line_height: 1500,
            premise_gap: 1500,
            literal_gap: 1000,
            label_gap: 200,
            margin: 300,
            stroke_width: 40,
            link_height: 600,
            node_radius: 380,
            text: "black".into(),
            line: "black".into(),
            par: "#0072b2".into(),
            link: "black".into(),
            highlight: "#d55e00".into(),
            background: None,
        }
    }
}

/// The character that stands for the raised `⊥` of a negated atom in the
/// text the notation writes, a control character that no atom name keeps.
const RAISED_BOT: char = '\u{1}';

/// The spelling of formulas and sequents as they are drawn: Unicode
/// connectives and mathematical italic letters, which [`run`] lays out.
const NOTATION: Notation = Notation {
    tensor: "⊗",
    par: "⅋",
    with: "&",
    plus: "⊕",
    lollipop: "⊸",
    bang: "!",
    quest: "?",
    one: "1",
    bot: "⊥",
    top: "⊤",
    zero: "0",
    dual: "\u{1}",
    turnstile: "⊢",
    align: "",
    atom: italic,
};

/// The spelling of formulas and sequents in a drawing's title: plain
/// Unicode text.
const PLAIN: Notation = Notation {
    dual: "⊥",
    atom: plain,
    ..NOTATION
};

/// Writes an atom's name with its Latin letters as mathematical italic
/// characters and its control characters, which XML cannot carry, as
/// `�`.
fn italic(out: &mut String, name: &str) {
    for c in name.chars() {
        out.push(match c {
            'h' => 'ℎ',
            'A'..='Z' => char::from_u32(0x1D434 + (c as u32 - 'A' as u32)).unwrap(),
            'a'..='z' => char::from_u32(0x1D44E + (c as u32 - 'a' as u32)).unwrap(),
            c if c.is_control() => '\u{FFFD}',
            c => c,
        });
    }
}

/// Writes an atom's name as it is, with its control characters as `�`.
fn plain(out: &mut String, name: &str) {
    out.extend(
        name.chars()
            .map(|c| if c.is_control() { '\u{FFFD}' } else { c }),
    );
}

/// Writes a character escaped for XML text and attribute values.
fn escape(out: &mut String, c: char) {
    match c {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        '"' => out.push_str("&quot;"),
        c => out.push(c),
    }
}

/// Returns a text escaped for XML.
fn escaped(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    text.chars().for_each(|c| escape(&mut out, c));
    out
}

/// A line of text laid out: the content of its `<text>` element and its
/// width.
#[derive(Clone, Debug, Default)]
struct Run {
    /// The escaped text, with a `<tspan>` for every superscript or
    /// subscript and for the text that returns to the baseline after one.
    markup: String,
    /// The width, in thousandths of an em of formula text.
    width: i64,
}

/// Lays out a line the notation or a rule name wrote, at `size`
/// thousandths of an em: [`RAISED_BOT`] becomes a superscript `⊥`, and
/// `₁` and `₂` subscript digits, since the font has no subscript
/// characters.
fn run(text: &str, size: i64) -> Run {
    let mut markup = String::with_capacity(text.len());
    // The width in millionths of an em of `size`, and how far the current
    // character sits above the baseline.
    let (mut width, mut raised) = (0, 0);
    let mut open = false;
    for c in text.chars() {
        let (c, script) = match c {
            RAISED_BOT => ('⊥', Some(RAISE)),
            '₁' => ('1', Some(-LOWER)),
            '₂' => ('2', Some(-LOWER)),
            c => (c, None),
        };
        if open && script.is_some() {
            markup.push_str("</tspan>");
            open = false;
        }
        match script {
            Some(shift) => {
                let target = shift * size / 1000;
                markup.push_str("<tspan");
                if raised != target {
                    write!(markup, r#" dy="{}""#, raised - target).unwrap();
                }
                write!(markup, r#" font-size="{}">"#, size * SCRIPT / 1000).unwrap();
                escape(&mut markup, c);
                markup.push_str("</tspan>");
                width += i64::from(advance(c)) * SCRIPT;
                raised = target;
            }
            None => {
                if raised != 0 {
                    write!(markup, r#"<tspan dy="{raised}">"#).unwrap();
                    open = true;
                    raised = 0;
                }
                escape(&mut markup, c);
                width += i64::from(advance(c)) * 1000;
            }
        }
    }
    if open {
        markup.push_str("</tspan>");
    }
    Run {
        markup,
        width: width * size / 1_000_000,
    }
}

/// Writes a `<text>` element with its left end at `x` and its baseline at
/// `y`, stretched or squeezed to the width the layout gave it; `attributes`
/// go into the start tag as they are.
fn text(out: &mut String, x: i64, y: i64, run: &Run, attributes: &str) {
    writeln!(
        out,
        r#"<text x="{x}" y="{y}" textLength="{}" lengthAdjust="spacing"{attributes}>{}</text>"#,
        run.width, run.markup
    )
    .unwrap();
}

/// Returns thousandths as a decimal number, without trailing zeros.
fn decimal(thousandths: i64) -> String {
    let text = format!("{}.{:03}", thousandths / 1000, thousandths % 1000);
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// Returns an SVG document of `width` by `height` thousandths of an em
/// with the given title and body, in the style's font and colours.
fn document(style: &Style, title: &str, width: i64, height: i64, body: &str) -> String {
    let px = |length: i64| decimal(length * i64::from(style.font_size));
    let mut out = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {width} {height}" font-family="'Euler Math', 'Neo Euler', serif" font-size="1000" fill="{}" xml:space="preserve">"#,
        px(width),
        px(height),
        escaped(&style.text),
    );
    write!(out, "\n<title>{}</title>\n", escaped(title)).unwrap();
    if let Some(background) = &style.background {
        writeln!(
            out,
            r#"<rect width="{width}" height="{height}" fill="{}"/>"#,
            escaped(background)
        )
        .unwrap();
    }
    out.push_str(body);
    out.push_str("</svg>");
    out
}

/// Returns a document that shows one line of text, the notation's `content`,
/// titled `title`.
fn line(style: &Style, title: &str, content: &str) -> String {
    let run = run(content, 1000);
    let margin = i64::from(style.margin);
    let mut body = String::new();
    text(&mut body, margin, margin + HEIGHT, &run, "");
    document(
        style,
        title,
        run.width + 2 * margin,
        HEIGHT + DEPTH + 2 * margin,
        &body,
    )
}

/// Returns a sequent one-sided, `⊢ A⊥, A`, as an SVG document of one line
/// of text, titled with the sequent in plain text.
pub fn sequent(sequent: &Sequent, style: &Style) -> String {
    let (mut drawn, mut title) = (String::new(), String::new());
    NOTATION.one_sided(&mut drawn, sequent);
    PLAIN.one_sided(&mut title, sequent);
    line(style, &title, &drawn)
}

/// Returns the sequent of an intuitionistic reading two-sided,
/// `A, A ⊸ B ⊢ B`, as an SVG document of one line of text, titled with
/// the sequent in plain text.
pub fn two_sided(reading: &Reading, style: &Style) -> String {
    let forest = reading.forest();
    let (mut drawn, mut title) = (String::new(), String::new());
    NOTATION.sequent(&mut drawn, forest, Some(reading), forest.roots(), false);
    PLAIN.sequent(&mut title, forest, Some(reading), forest.roots(), false);
    line(style, &title, &drawn)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A raised `⊥` and a subscript are smaller, shifted, and shifted back
    /// for what follows; the width counts them at their size.
    #[test]
    fn scripts_shift_and_return() {
        let r = run("A\u{1}, A", 1000);
        assert_eq!(
            r.markup,
            r#"A<tspan dy="-400" font-size="700">⊥</tspan><tspan dy="400">, A</tspan>"#
        );
        let (a, bot, comma, space) = (770, 874, 277, 333);
        assert_eq!(r.width, a + bot * 7 / 10 + comma + space + a);

        let r = run("&L₁", 800);
        assert_eq!(
            r.markup,
            r#"&amp;L<tspan dy="120" font-size="560">1</tspan>"#
        );
    }

    /// Atom names keep their letters as math italic and lose their control
    /// characters.
    #[test]
    fn names_are_italic() {
        let mut out = String::new();
        italic(&mut out, "Ah_1α\u{1}");
        assert_eq!(out, "𝐴ℎ_1α\u{FFFD}");
    }
}
