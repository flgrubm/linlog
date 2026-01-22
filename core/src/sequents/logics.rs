// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use std::convert::Infallible;

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for Infallible {}
impl sealed::Sealed for () {}

pub trait UnitOrInfallible:
    sealed::Sealed + Copy + Clone + std::fmt::Debug + PartialEq + Eq
{
}

impl<T: sealed::Sealed + Copy + Clone + std::fmt::Debug + PartialEq + Eq> UnitOrInfallible for T {}

pub trait LogicPayload: Copy + Clone + std::fmt::Debug + PartialEq + Eq {}
impl<T: Copy + Clone + std::fmt::Debug + PartialEq + Eq> LogicPayload for T {}

pub trait Logic {
    type Mult: UnitOrInfallible;
    type Add: UnitOrInfallible;
    type Exp: UnitOrInfallible;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LL;

impl Logic for LL {
    type Mult = ();
    type Add = ();
    type Exp = ();
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MELL;

impl Logic for MELL {
    type Mult = ();
    type Add = Infallible;
    type Exp = ();
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MLL;

impl Logic for MLL {
    type Mult = ();
    type Add = Infallible;
    type Exp = Infallible;
}

#[derive(Copy, Clone, Debug)]
pub struct MALL;

impl Logic for MALL {
    type Mult = ();
    type Add = ();
    type Exp = Infallible;
}

#[derive(Copy, Clone, Debug)]
pub struct ALL;

impl Logic for ALL {
    type Mult = Infallible;
    type Add = ();
    type Exp = Infallible;
}

trait IsSubSetLogic<L: Logic> {
    const B: bool;
}
