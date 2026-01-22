// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use condtype::CondType;
use std::convert::Infallible;

type UnitOrInfallable<const ACTIVATE: bool> = CondType<ACTIVATE, (), Infallible>;

mod sealed {
    pub trait Sealed {}
}

use sealed::Sealed;

impl Sealed for Infallible {}
impl Sealed for () {}

pub trait UnitOrInfallibleT:
    sealed::Sealed + Copy + Clone + std::fmt::Debug + PartialEq + Eq
{
}

impl<T: sealed::Sealed + Copy + Clone + std::fmt::Debug + PartialEq + Eq> UnitOrInfallibleT for T {}

pub trait LogicPayload: Copy + Clone + std::fmt::Debug + PartialEq + Eq {}
impl<T: Copy + Clone + std::fmt::Debug + PartialEq + Eq> LogicPayload for T {}

pub trait Logic: sealed::Sealed + Default {
    type Mult: UnitOrInfallibleT;
    type Add: UnitOrInfallibleT;
    type Exp: UnitOrInfallibleT;

    const MULT: bool;
    const ADD: bool;
    const EXP: bool;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct LL;

impl Sealed for LL {}

impl Logic for LL {
    type Mult = UnitOrInfallable<{ <LL as Logic>::MULT }>;
    type Add = UnitOrInfallable<{ <LL as Logic>::ADD }>;
    type Exp = UnitOrInfallable<{ <LL as Logic>::EXP }>;

    const MULT: bool = true;
    const ADD: bool = true;
    const EXP: bool = true;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct MELL;

impl Sealed for MELL {}

impl Logic for MELL {
    type Mult = UnitOrInfallable<{ <MELL as Logic>::MULT }>;
    type Add = UnitOrInfallable<{ <MELL as Logic>::ADD }>;
    type Exp = UnitOrInfallable<{ <MELL as Logic>::EXP }>;

    const MULT: bool = true;
    const ADD: bool = false;
    const EXP: bool = true;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct MLL;

impl Sealed for MLL {}

impl Logic for MLL {
    type Mult = UnitOrInfallable<{ <MLL as Logic>::MULT }>;
    type Add = UnitOrInfallable<{ <MLL as Logic>::ADD }>;
    type Exp = UnitOrInfallable<{ <MLL as Logic>::EXP }>;

    const MULT: bool = true;
    const ADD: bool = false;
    const EXP: bool = false;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct MALL;

impl Sealed for MALL {}

impl Logic for MALL {
    type Mult = UnitOrInfallable<{ <MALL as Logic>::MULT }>;
    type Add = UnitOrInfallable<{ <MALL as Logic>::ADD }>;
    type Exp = UnitOrInfallable<{ <MALL as Logic>::EXP }>;

    const MULT: bool = true;
    const ADD: bool = true;
    const EXP: bool = false;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct ALL;

impl Sealed for ALL {}

impl Logic for ALL {
    type Mult = UnitOrInfallable<{ <ALL as Logic>::MULT }>;
    type Add = UnitOrInfallable<{ <ALL as Logic>::ADD }>;
    type Exp = UnitOrInfallable<{ <ALL as Logic>::EXP }>;

    const MULT: bool = false;
    const ADD: bool = true;
    const EXP: bool = false;
}

pub const fn is_subset_logic<LSub: Logic, LSup: Logic>() -> bool {
    (!LSub::MULT || LSup::MULT) && (!LSub::ADD || LSup::ADD) && (!LSub::EXP || LSup::EXP)
}

mod test {
    use super::*;
    #[test]
    fn subset_logic_test() {
        assert!(is_subset_logic::<MLL, LL>());
        assert!(is_subset_logic::<MLL, LL>());
        assert!(is_subset_logic::<MLL, LL>());
        assert!(is_subset_logic::<MLL, LL>());
        assert!(is_subset_logic::<MLL, LL>());

        assert!(!is_subset_logic::<LL, MLL>());
        assert!(is_subset_logic::<MLL, MLL>());
        assert!(!is_subset_logic::<MELL, MLL>());
        assert!(!is_subset_logic::<ALL, MLL>());
        assert!(!is_subset_logic::<MALL, MLL>());

        assert!(!is_subset_logic::<LL, MELL>());
        assert!(is_subset_logic::<MLL, MELL>());
        assert!(is_subset_logic::<MELL, MELL>());
        assert!(!is_subset_logic::<ALL, MELL>());
        assert!(!is_subset_logic::<MALL, MELL>());

        assert!(!is_subset_logic::<LL, ALL>());
        assert!(!is_subset_logic::<MLL, ALL>());
        assert!(!is_subset_logic::<MELL, ALL>());
        assert!(is_subset_logic::<ALL, ALL>());
        assert!(!is_subset_logic::<MALL, ALL>());

        assert!(!is_subset_logic::<LL, MALL>());
        assert!(is_subset_logic::<MLL, MALL>());
        assert!(!is_subset_logic::<MELL, MALL>());
        assert!(is_subset_logic::<ALL, MALL>());
        assert!(is_subset_logic::<MALL, MALL>());
    }
}
