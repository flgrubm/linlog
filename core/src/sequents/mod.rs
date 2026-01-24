// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

pub mod logics;
pub mod parsing;
mod serialize;
pub mod terms;

use logics::{LL, Logic};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::iter::zip;

use terms::Expression;

#[derive(Clone, Debug, Default)]
pub struct Sequent<L: Logic> {
    term_arena: Vec<Expression<L>>,
    term_ids: Vec<usize>,
    variable_dict: Vec<String>,
}

impl Serialize for Sequent<LL> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let proxy = serialize::Sequent::from(self.clone());
        proxy.serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for Sequent<LL> {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let proxy = serialize::Sequent::deserialize(deserializer)?;
        Sequent::try_from(proxy).map_err(serde::de::Error::custom)
    }
}

impl<L: Logic> Sequent<L> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check whether the internal data structure is correct
    pub fn verify_integrity(&self) -> Result<(), crate::Error> {
        let num_vars = self.variable_dict.len();
        let num_terms = self.term_arena.len();

        // check that terms only reference
        //   - other terms with lower IDs than themselves
        //   - existint variable IDS
        self.term_arena
            .iter()
            .enumerate()
            .try_for_each(|(n, e)| e.check_bounds(num_vars, n))?;

        // check that all term indices are valid
        self.term_ids.iter().try_for_each(|n| {
            if *n >= num_terms {
                Err(crate::Error::TermIndexOutOfBounds(*n, num_terms))
            } else {
                Ok(())
            }
        })?;

        Ok(())
    }

    fn optimize_term_ids(&mut self) -> Result<(), crate::Error> {
        self.term_ids.sort();
        self.term_ids.shrink_to_fit();
        let n = self.term_ids.last().unwrap();
        let num_terms = self.term_arena.len();
        if *n >= num_terms {
            Err(crate::Error::TermIndexOutOfBounds(*n, num_terms))
        } else {
            Ok(())
        }
    }

    fn optimize_term_arena(&mut self) -> Result<(), crate::Error> {
        use Expression::*;

        // determine cleanup measures and check integrity

        let num_terms = self.term_arena.len();

        for n in self.term_ids.iter() {
            if *n >= num_terms {
                return Err(crate::Error::TermIndexOutOfBounds(*n, num_terms));
            }
        }

        let mut reachable = vec![false; num_terms];
        let mut visit_stack = self.term_ids.clone();

        while let Some(n) = visit_stack.pop() {
            if !reachable[n] {
                reachable[n] = true;
                match self.term_arena[n] {
                    MultOp(_, _, k, l) | AddOp(_, _, k, l) => {
                        if k >= n {
                            return Err(crate::Error::SubtermIndexNotDecreasing(k, n));
                        }
                        if l >= n {
                            return Err(crate::Error::SubtermIndexNotDecreasing(l, n));
                        }
                        if !reachable[k] {
                            visit_stack.push(k);
                        }
                        if !reachable[l] {
                            visit_stack.push(l);
                        }
                    }
                    ExpOp(_, _, k) => {
                        if k >= n {
                            return Err(crate::Error::SubtermIndexNotDecreasing(k, n));
                        }
                        if !reachable[k] {
                            reachable[k] = true;
                            visit_stack.push(k);
                        }
                    }
                    _ => {}
                }
            }
        }

        #[derive(Copy, Clone, Debug)]
        enum TermState {
            Remain,
            Unreachable,
            DuplicateOf(usize),
        }

        use TermState::*;

        let mut hm = HashMap::<Expression<L>, usize>::new();
        let mut states = Vec::<TermState>::with_capacity(num_terms);

        for (n, (is_reachable, e)) in zip(reachable.into_iter(), self.term_arena.iter()).enumerate()
        {
            if is_reachable {
                if let Some(k) = hm.get(e) {
                    states.push(DuplicateOf(*k));
                } else {
                    hm.insert(*e, n);
                    states.push(Remain);
                }
            } else {
                states.push(Unreachable);
            }
        }

        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        enum RedirectState {
            Remove,
            MoveTo(usize),
            RemoveAndPointTo(usize),
        }

        use RedirectState::*;

        let mut fresh_index = 0usize;
        let mut redirects = Vec::<RedirectState>::with_capacity(num_terms);

        for s in states.into_iter() {
            match s {
                Unreachable => redirects.push(Remove),
                Remain => {
                    redirects.push(MoveTo(fresh_index));
                    fresh_index += 1;
                }
                DuplicateOf(n) => {
                    if let MoveTo(k) = redirects[n] {
                        redirects.push(RemoveAndPointTo(k));
                    } else {
                        unreachable!()
                    }
                }
            }
        }

        // execute cleanup

        let where_to_find = |k: &usize| {
            debug_assert!(*k < redirects.len());
            debug_assert!(redirects[*k] != Remove);

            match redirects[*k] {
                MoveTo(n) => n,
                RemoveAndPointTo(n) => {
                    if let MoveTo(m) = redirects[n] {
                        m
                    } else {
                        unreachable!()
                    }
                }
                _ => {
                    unreachable!()
                }
            }
        };

        let apply_cleanup_terms = |(state, e): (&RedirectState, &Expression<L>)| {
            if let MoveTo(_) = state {
                match *e {
                    MultOp(marker, b, m, n) => {
                        let m_new = where_to_find(&m);
                        let n_new = where_to_find(&n);
                        Some(MultOp(marker, b, m_new, n_new))
                    }
                    AddOp(marker, b, m, n) => {
                        let m_new = where_to_find(&m);
                        let n_new = where_to_find(&n);
                        Some(AddOp(marker, b, m_new, n_new))
                    }
                    ExpOp(marker, b, m) => {
                        let m_new = where_to_find(&m);
                        Some(ExpOp(marker, b, m_new))
                    }
                    e => Some(e),
                }
            } else {
                None
            }
        };

        self.term_arena = zip(redirects.iter(), self.term_arena.iter())
            .filter_map(apply_cleanup_terms)
            .collect();

        self.term_ids = self.term_ids.iter().map(where_to_find).collect();

        Ok(())
    }

    fn optimize_variable_dict(&mut self) -> Result<(), crate::Error> {
        use Expression::*;
        let num_variables = self.variable_dict.len();
        let mut var_dict_new = Vec::<String>::with_capacity(num_variables);
        let mut hm = HashMap::<String, usize>::with_capacity(num_variables);

        for e in self.term_arena.iter_mut() {
            if let Var(b, n) = *e {
                let name_ref: &str = self
                    .variable_dict
                    .get(n)
                    .ok_or(crate::Error::InvalidVariableIndex(n, num_variables))?
                    .as_ref();
                if let Some(k) = hm.get(name_ref) {
                    *e = Var(b, *k);
                } else {
                    let fresh_variable_id = var_dict_new.len();
                    let name = name_ref.to_string();
                    hm.insert(name.clone(), fresh_variable_id);
                    var_dict_new.push(name);
                }
            }
        }
        var_dict_new.shrink_to_fit();
        self.variable_dict = var_dict_new;
        Ok(())
    }

    /// Remove unreachable terms and collapse duplicate items
    /// This is rather inefficient, so use only if necessary
    pub fn optimize(&mut self) -> Result<(), crate::Error> {
        self.optimize_term_arena()?;
        self.optimize_term_ids()?;
        self.optimize_variable_dict()?;
        Ok(())
    }

    /// Consume another sequent and add its terms to self
    pub fn add(&mut self, s: Sequent<L>) {
        let offset_variables = self.variable_dict.len();
        let offset_terms = self.term_arena.len();

        self.term_arena.extend(
            s.term_arena
                .into_iter()
                .map(|e| e.offset(offset_variables, offset_terms)),
        );

        self.term_ids
            .extend(s.term_ids.into_iter().map(|n| n + offset_terms));

        self.variable_dict.extend(s.variable_dict);
    }
}

impl<'a> From<(parsing::Term<'a>, bool)> for Sequent<LL> {
    fn from((t, polarity): (parsing::Term<'a>, bool)) -> Sequent<LL> {
        fn recursion_helper<'a>(
            t: parsing::Term<'a>,
            polarity: bool,
            term_arena: &mut Vec<Expression<LL>>,
            variable_dict: &mut Vec<String>,
        ) -> usize {
            use Expression::*;
            use parsing::Term as T;
            if let T::Dual(nt) = t {
                recursion_helper(*nt, !polarity, term_arena, variable_dict)
            } else {
                let e = match t {
                    T::Var(s) => {
                        let var_index = variable_dict.len();
                        variable_dict.push(s.to_string());
                        Var(true, var_index)
                    }
                    T::One => MultConst((), true),
                    T::Bot => MultConst((), false),
                    T::Top => AddConst((), true),
                    T::Zero => AddConst((), false),
                    T::Bang(nt) => {
                        let n = recursion_helper(*nt, polarity, term_arena, variable_dict);
                        ExpOp((), true, n)
                    }
                    T::Quest(nt) => {
                        let n = recursion_helper(*nt, polarity, term_arena, variable_dict);
                        ExpOp((), false, n)
                    }
                    T::Tensor(nt, mt) => {
                        let n = recursion_helper(*nt, polarity, term_arena, variable_dict);
                        let m = recursion_helper(*mt, polarity, term_arena, variable_dict);
                        MultOp((), true, n, m)
                    }
                    T::Par(nt, mt) => {
                        let n = recursion_helper(*nt, polarity, term_arena, variable_dict);
                        let m = recursion_helper(*mt, polarity, term_arena, variable_dict);
                        MultOp((), false, n, m)
                    }
                    T::With(nt, mt) => {
                        let n = recursion_helper(*nt, polarity, term_arena, variable_dict);
                        let m = recursion_helper(*mt, polarity, term_arena, variable_dict);
                        AddOp((), true, n, m)
                    }
                    T::Plus(nt, mt) => {
                        let n = recursion_helper(*nt, polarity, term_arena, variable_dict);
                        let m = recursion_helper(*mt, polarity, term_arena, variable_dict);
                        AddOp((), false, n, m)
                    }
                    T::Lollipop(nt, mt) => {
                        // Lollipop is a Par where the first element has its polarity inverted
                        let n = recursion_helper(*nt, !polarity, term_arena, variable_dict);
                        let m = recursion_helper(*mt, polarity, term_arena, variable_dict);
                        MultOp((), false, n, m)
                    }
                    T::Dual(nt) => unreachable!(),
                };

                let index = term_arena.len();
                if !polarity {
                    let e = e.invert();
                }
                term_arena.push(e);
                index
            }
        }

        let mut term_arena = Vec::<Expression<LL>>::new();
        let mut variable_dict = Vec::<String>::new();
        let index = recursion_helper(t, polarity, &mut term_arena, &mut variable_dict);

        Sequent {
            term_arena,
            term_ids: vec![index],
            variable_dict,
        }
    }
}

impl<'a> From<parsing::Sequent<'a>> for Sequent<LL> {
    fn from(s: parsing::Sequent<'a>) -> Sequent<LL> {
        let lhs_terms = s.left.into_iter().map(|t| (t, false));
        let rhs_terms = s.right.into_iter().map(|t| (t, true));
        let mut sequent = Sequent::new();
        lhs_terms
            .chain(rhs_terms)
            .map(Sequent::from)
            .for_each(|s| sequent.add(s));
        sequent.optimize().unwrap();
        sequent
    }
}

impl std::str::FromStr for Sequent<LL> {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Sequent<LL>, Self::Err> {
        Ok(Sequent::from(parsing::Sequent::try_from(s)?))
    }
}
