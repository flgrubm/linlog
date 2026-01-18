// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

#[derive(Debug, Copy, Clone)]
pub enum Symbol {
    Var(usize),
    Zero,
    One,
    Bot,
    Top,
    Dual,
    Bang,
    Quest,
    Tensor,
    Par,
    With,
    Plus,
    Lollipop,
}

#[derive(Debug)]
pub struct Term {
    term: Box<[Symbol]>,
}

#[derive(Debug)]
struct TermView<'a> {
    tv: &'a [Symbol],
}

#[derive(Debug)]
enum SymbolView<'a> {
    Var(usize),
    Zero,
    One,
    Bot,
    Top,
    Dual(TermView<'a>),
    Bang(TermView<'a>),
    Quest(TermView<'a>),
    Tensor(TermView<'a>, TermView<'a>),
    Par(TermView<'a>, TermView<'a>),
    With(TermView<'a>, TermView<'a>),
    Plus(TermView<'a>, TermView<'a>),
    Lollipop(TermView<'a>, TermView<'a>),
}

impl<'a> TermView<'a> {
    fn get_subterm_split_index(&'a self) -> usize {
        let mut leaves_left = 1usize;
        let mut it = self.tv.iter().enumerate();
        let _h = it.next(); // the splitting symbol should be ignored

        use Symbol::*;

        for (i, sym) in it {
            match sym {
                Var(_) | Zero | One | Bot | Top => {
                    if leaves_left == 1 {
                        return i + 1;
                    } else {
                        leaves_left -= 1
                    }
                }
                Dual | Bang | Quest => {}
                Tensor | Par | With | Plus | Lollipop => {
                    leaves_left += 1;
                }
            }
        }
        unreachable!()
    }

    fn get_subterm_view(&'a self) -> SymbolView<'a> {
        debug_assert!(!self.tv.is_empty());
        use Symbol::*;
        use SymbolView as SV;
        match self.tv[0] {
            Var(i) => SV::Var(i),
            Zero => SV::Zero,
            One => SV::One,
            Bot => SV::Bot,
            Top => SV::Top,
            Dual => SV::Dual(TermView { tv: &self.tv[1..] }),
            Bang => SV::Bang(TermView { tv: &self.tv[1..] }),
            Quest => SV::Quest(TermView { tv: &self.tv[1..] }),
            Tensor => {
                let i = self.get_subterm_split_index();
                SV::Tensor(
                    TermView { tv: &self.tv[1..i] },
                    TermView { tv: &self.tv[i..] },
                )
            }
            Par => {
                let i = self.get_subterm_split_index();
                SV::Par(
                    TermView { tv: &self.tv[1..i] },
                    TermView { tv: &self.tv[i..] },
                )
            }
            With => {
                let i = self.get_subterm_split_index();
                SV::With(
                    TermView { tv: &self.tv[1..i] },
                    TermView { tv: &self.tv[i..] },
                )
            }
            Plus => {
                let i = self.get_subterm_split_index();
                SV::Plus(
                    TermView { tv: &self.tv[1..i] },
                    TermView { tv: &self.tv[i..] },
                )
            }
            Lollipop => {
                let i = self.get_subterm_split_index();
                SV::Lollipop(
                    TermView { tv: &self.tv[1..i] },
                    TermView { tv: &self.tv[i..] },
                )
            }
        }
    }

    fn write_to_string<const IS_OUTER_MOST: bool>(
        &'a self,
        s: &'a mut String,
        labels: &'a [String],
    ) {
        use SymbolView::*;
        match self.get_subterm_view() {
            Var(i) => s.push_str(&labels[i]),
            Zero => s.push('0'),
            One => s.push('1'),
            Bot => s.push('⊥'),
            Top => s.push('⊤'),
            Dual(t) => {
                s.push('~');
                t.write_to_string::<false>(s, labels);
            }
            Bang(t) => {
                s.push('!');
                t.write_to_string::<false>(s, labels);
            }
            Quest(t) => {
                s.push('?');
                t.write_to_string::<false>(s, labels);
            }
            Tensor(t1, t2) => {
                if !IS_OUTER_MOST {
                    s.push('(');
                }
                t1.write_to_string::<false>(s, labels);
                s.push_str(" ⊗ ");
                t2.write_to_string::<false>(s, labels);
                if !IS_OUTER_MOST {
                    s.push(')');
                }
            }
            Par(t1, t2) => {
                if !IS_OUTER_MOST {
                    s.push('(');
                }
                t1.write_to_string::<false>(s, labels);
                s.push_str(" ⅋ ");
                t2.write_to_string::<false>(s, labels);
                if !IS_OUTER_MOST {
                    s.push(')');
                }
            }
            With(t1, t2) => {
                if !IS_OUTER_MOST {
                    s.push('(');
                }
                t1.write_to_string::<false>(s, labels);
                s.push_str(" & ");
                t2.write_to_string::<false>(s, labels);
                if !IS_OUTER_MOST {
                    s.push(')');
                }
            }
            Plus(t1, t2) => {
                if !IS_OUTER_MOST {
                    s.push('(');
                }
                t1.write_to_string::<false>(s, labels);
                s.push_str(" ⊕ ");
                t2.write_to_string::<false>(s, labels);
                if !IS_OUTER_MOST {
                    s.push(')');
                }
            }
            Lollipop(t1, t2) => {
                if !IS_OUTER_MOST {
                    s.push('(');
                }
                t1.write_to_string::<false>(s, labels);
                s.push_str(" ⊸ ");
                t2.write_to_string::<false>(s, labels);
                if !IS_OUTER_MOST {
                    s.push(')');
                }
            }
        }
    }
}

impl<'a> Term {
    pub unsafe fn new_unsafe(term: Box<[Symbol]>) -> Self {
        Self { term }
    }

    fn into_view(&'a self) -> TermView<'a> {
        TermView { tv: &self.term }
    }
}

#[derive(Debug)]
pub struct Sequent {
    pub left: Box<[Term]>,
    pub right: Box<[Term]>,
    pub labels: Box<[String]>,
}

impl std::fmt::Display for Sequent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = String::new();

        fn add_term_vec(s: &mut String, terms: &[Term], labels: &[String]) {
            debug_assert!(!terms.is_empty());
            let mut it = terms.iter();
            it.next()
                .unwrap() // element exists since terms is not empty
                .into_view()
                .write_to_string::<true>(s, labels);
            for t in it {
                s.push_str(", ");
                t.into_view().write_to_string::<true>(s, labels);
            }
        }

        if !self.left.is_empty() {
            add_term_vec(&mut s, &self.left, &self.labels);
            s.push(' ');
        }

        s.push('⊢');

        if !self.right.is_empty() {
            s.push(' ');
            add_term_vec(&mut s, &self.right, &self.labels);
        }
        write!(f, "{}", s)
    }
}
