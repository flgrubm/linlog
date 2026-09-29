// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What the layout knows of the Euler Math font (version 0.75, 1000 units
//! per em): the advance of every character the drawings set, and the
//! vertical metrics of a line. The advances were read from the font's
//! `hmtx` table once; nothing is measured at run time.

/// The advance width of every character the drawings set, in thousandths
/// of an em, sorted by character: printable ASCII, the mathematical italic
/// Latin letters, Greek, and the connectives, units, turnstile and
/// vertical dots.
const ADVANCES: [(char, u16); 203] = [
    (' ', 333),
    ('!', 295),
    ('"', 214),
    ('#', 500),
    ('$', 557),
    ('%', 840),
    ('&', 737),
    ('\'', 212),
    ('(', 328),
    (')', 328),
    ('*', 277),
    ('+', 755),
    (',', 277),
    ('-', 544),
    ('.', 277),
    ('/', 501),
    ('0', 499),
    ('1', 499),
    ('2', 499),
    ('3', 499),
    ('4', 499),
    ('5', 499),
    ('6', 499),
    ('7', 499),
    ('8', 499),
    ('9', 499),
    (':', 216),
    (';', 216),
    ('<', 800),
    ('=', 756),
    ('>', 800),
    ('?', 362),
    ('@', 744),
    ('A', 770),
    ('B', 655),
    ('C', 714),
    ('D', 828),
    ('E', 604),
    ('F', 499),
    ('G', 765),
    ('H', 783),
    ('I', 394),
    ('J', 402),
    ('K', 668),
    ('L', 559),
    ('M', 1044),
    ('N', 829),
    ('O', 803),
    ('P', 576),
    ('Q', 828),
    ('R', 609),
    ('S', 557),
    ('T', 514),
    ('U', 774),
    ('V', 646),
    ('W', 986),
    ('X', 666),
    ('Y', 555),
    ('Z', 666),
    ('[', 290),
    ('\\', 501),
    (']', 290),
    ('^', 499),
    ('_', 450),
    ('`', 201),
    ('a', 609),
    ('b', 588),
    ('c', 486),
    ('d', 603),
    ('e', 499),
    ('f', 419),
    ('g', 568),
    ('h', 621),
    ('i', 360),
    ('j', 331),
    ('k', 555),
    ('l', 365),
    ('m', 915),
    ('n', 664),
    ('o', 563),
    ('p', 589),
    ('q', 605),
    ('r', 432),
    ('s', 455),
    ('t', 416),
    ('u', 642),
    ('v', 495),
    ('w', 812),
    ('x', 526),
    ('y', 593),
    ('z', 470),
    ('{', 345),
    ('|', 213),
    ('}', 345),
    ('~', 551),
    ('Α', 770),
    ('Β', 655),
    ('Γ', 428),
    ('Δ', 713),
    ('Ε', 604),
    ('Ζ', 666),
    ('Η', 783),
    ('Θ', 757),
    ('Ι', 394),
    ('Κ', 668),
    ('Λ', 770),
    ('Μ', 1044),
    ('Ν', 829),
    ('Ξ', 596),
    ('Ο', 803),
    ('Π', 722),
    ('Ρ', 576),
    ('Σ', 646),
    ('Τ', 492),
    ('Υ', 716),
    ('Φ', 833),
    ('Χ', 666),
    ('Ψ', 703),
    ('Ω', 875),
    ('α', 658),
    ('β', 662),
    ('γ', 608),
    ('δ', 501),
    ('ε', 486),
    ('ζ', 512),
    ('η', 560),
    ('θ', 554),
    ('ι', 334),
    ('κ', 555),
    ('λ', 541),
    ('μ', 617),
    ('ν', 599),
    ('ξ', 553),
    ('ο', 563),
    ('π', 609),
    ('ρ', 548),
    ('σ', 605),
    ('τ', 513),
    ('υ', 587),
    ('φ', 763),
    ('χ', 576),
    ('ψ', 754),
    ('ω', 851),
    ('ℎ', 621),
    ('⅋', 737),
    ('⊕', 668),
    ('⊗', 668),
    ('⊢', 698),
    ('⊤', 874),
    ('⊥', 874),
    ('⊸', 1010),
    ('⋮', 768),
    ('𝐴', 770),
    ('𝐵', 655),
    ('𝐶', 714),
    ('𝐷', 828),
    ('𝐸', 604),
    ('𝐹', 499),
    ('𝐺', 765),
    ('𝐻', 783),
    ('𝐼', 394),
    ('𝐽', 402),
    ('𝐾', 668),
    ('𝐿', 559),
    ('𝑀', 1044),
    ('𝑁', 829),
    ('𝑂', 803),
    ('𝑃', 576),
    ('𝑄', 828),
    ('𝑅', 609),
    ('𝑆', 557),
    ('𝑇', 492),
    ('𝑈', 774),
    ('𝑉', 646),
    ('𝑊', 986),
    ('𝑋', 666),
    ('𝑌', 555),
    ('𝑍', 666),
    ('𝑎', 609),
    ('𝑏', 588),
    ('𝑐', 486),
    ('𝑑', 603),
    ('𝑒', 499),
    ('𝑓', 419),
    ('𝑔', 568),
    ('𝑖', 360),
    ('𝑗', 331),
    ('𝑘', 555),
    ('𝑙', 365),
    ('𝑚', 915),
    ('𝑛', 664),
    ('𝑜', 563),
    ('𝑝', 589),
    ('𝑞', 605),
    ('𝑟', 432),
    ('𝑠', 455),
    ('𝑡', 416),
    ('𝑢', 642),
    ('𝑣', 495),
    ('𝑤', 812),
    ('𝑥', 526),
    ('𝑦', 593),
    ('𝑧', 470),
];

/// The advance of a character outside the table, in thousandths of an em:
/// a viewer sets it in some other font, and `textLength` makes the line
/// fit whatever that font's width is.
const FALLBACK: u32 = 650;

/// How far the highest character of a line reaches above the baseline, a
/// raised `⊥` of a negated atom, in thousandths of an em.
pub(super) const HEIGHT: i64 = 890;

/// How far the lowest character of a line reaches below the baseline, the
/// comma, in thousandths of an em.
pub(super) const DEPTH: i64 = 210;

/// The height of the math axis above the baseline, where `⊗`, `⊸` and
/// the inference lines sit, in thousandths of an em.
pub(super) const AXIS: i64 = 250;

/// The size of a superscript or subscript, in thousandths of the size of
/// the text it belongs to.
pub(super) const SCRIPT: i64 = 700;

/// How far a superscript's baseline is raised, in thousandths of an em of
/// the text it belongs to.
pub(super) const RAISE: i64 = 400;

/// How far a subscript's baseline is lowered, in thousandths of an em of
/// the text it belongs to.
pub(super) const LOWER: i64 = 150;

/// Returns the advance of a character, in thousandths of an em.
pub(super) fn advance(c: char) -> u32 {
    match ADVANCES.binary_search_by_key(&c, |&(d, _)| d) {
        Ok(i) => u32::from(ADVANCES[i].1),
        Err(_) => FALLBACK,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table is sorted, so that the binary search finds every entry.
    #[test]
    fn advances_are_sorted() {
        assert!(ADVANCES.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(advance('⊗'), 668);
        assert_eq!(advance('\u{FFFD}'), FALLBACK);
    }
}
