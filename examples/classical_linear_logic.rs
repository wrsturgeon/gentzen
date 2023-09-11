/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Abstract syntax tree for linear logic with sequent-calculus proof search built in.

#![deny(warnings)]
#![allow(clippy::needless_borrowed_reference)]

use gentzen::{sequents::RhsOnlyWithExchange, Infer, Inference, Multiset, Sequent};
use std::{collections::BTreeSet, rc::Rc};

#[cfg(test)]
use gentzen::{prove, Error};

fn main() {}

/// Abstract syntax tree for linear logic with sequent-calculus proof search built in.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Ast {
    /// Unit for multiplicative conjunction.
    One,
    /// Unit for multiplicative disjunction.
    Bottom,
    /// Unit for additive conjunction.
    Top,
    /// Unit for additive disjunction.
    Zero,
    /// Raw value identified by number (for efficient comparison).
    Value(usize),
    /// The "of course" exponential.
    Bang(Box<Self>),
    /// The "why not" exponential.
    Quest(Box<Self>),
    /// Dual, i.e. linear negation.
    Dual(Box<Self>),
    /// Multiplicative conjunction.
    Times(Box<Self>, Box<Self>),
    /// Multiplicative disjunction.
    Par(Box<Self>, Box<Self>),
    /// Additive conjunction.
    With(Box<Self>, Box<Self>),
    /// Additive disjunction.
    Plus(Box<Self>, Box<Self>),
}

impl core::fmt::Display for Ast {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::One => write!(f, "1"),
            Self::Bottom => write!(f, "\u{22a5}"),
            Self::Top => write!(f, "\u{22a4}"),
            Self::Zero => write!(f, "0"),
            Self::Value(i) => write!(f, "P{i}"),
            Self::Bang(ref arg) => write!(f, "!{arg}"),
            Self::Quest(ref arg) => write!(f, "?{arg}"),
            Self::Dual(ref arg) => write!(f, "~{arg}"),
            Self::Times(ref lhs, ref rhs) => write!(f, "({lhs} \u{2297} {rhs})"),
            Self::Par(ref lhs, ref rhs) => write!(f, "({lhs} \u{214b} {rhs})"),
            Self::With(ref lhs, ref rhs) => write!(f, "({lhs} & {rhs})"),
            Self::Plus(ref lhs, ref rhs) => write!(f, "({lhs} \u{2295} {rhs})"),
        }
    }
}

/// The "of course" exponential.
#[must_use]
#[inline(always)]
pub fn bang(arg: Ast) -> Ast {
    Ast::Bang(Box::new(arg))
}

/// The "why not" exponential.
#[must_use]
#[inline(always)]
pub fn quest(arg: Ast) -> Ast {
    Ast::Quest(Box::new(arg))
}

impl Ast {
    /// Par operator, since it's a pain in the ass to type.
    #[must_use]
    #[inline(always)]
    pub fn par(self, rhs: Self) -> Self {
        Self::Par(Box::new(self), Box::new(rhs))
    }
}

impl core::ops::Mul<Self> for Ast {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::Times(Box::new(self), Box::new(rhs))
    }
}

impl core::ops::BitAnd<Self> for Ast {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self::With(Box::new(self), Box::new(rhs))
    }
}

impl core::ops::Add<Self> for Ast {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self::Plus(Box::new(self), Box::new(rhs))
    }
}

impl core::ops::Sub<Self> for Ast {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        // Rule that "A -* B" === "A^T par B"
        #[allow(clippy::arithmetic_side_effects)]
        (-self).par(rhs)
    }
}

impl core::ops::Neg for Ast {
    type Output = Self;
    #[inline(always)]
    #[allow(clippy::arithmetic_side_effects)]
    fn neg(self) -> Self::Output {
        Self::Dual(Box::new(self))
    }
}

// FIXME: does not seem right that you have to write `vec![below.qed()]`, since QED is all that matters
impl Infer<RhsOnlyWithExchange<Self>> for Ast {
    #[inline]
    fn above(
        &self,
        context: RhsOnlyWithExchange<Self>,
        below: &Rc<RhsOnlyWithExchange<Self>>,
    ) -> BTreeSet<Inference<RhsOnlyWithExchange<Self>>> {
        if context.rhs().contains(&Ast::Top)
            || context.rhs() == &core::iter::once(Self::Dual(Box::new(self.clone()))).collect()
        {
            return BTreeSet::from_iter([below.require([])]);
        }
        match *self {
            Self::Top => BTreeSet::from_iter([below.require([])]),
            Self::One if context.is_empty() => BTreeSet::from_iter([below.require([])]),
            Self::Bang(ref arg) if matches!(context.only(), Some(&Self::Quest(_))) => {
                BTreeSet::from_iter([below.require([context.with([arg.as_ref().clone()])])])
            }
            Self::One | Self::Zero | Self::Value(_) | Self::Bang(_) => BTreeSet::new(),
            Self::Bottom => BTreeSet::from_iter([below.require([context])]),
            Self::Quest(ref arg) => BTreeSet::from_iter([
                below.require([context.clone()]),
                below.require([context.with([arg.as_ref().clone()])]),
                below.require([context.with([Self::Quest(arg.clone()), Self::Quest(arg.clone())])]),
            ]),
            Self::Dual(ref dual) => {
                BTreeSet::from_iter([below.require([context.with([match *dual.as_ref() {
                    Self::One => Self::Bottom,
                    Self::Bottom => Self::One,
                    Self::Top => Self::Zero,
                    Self::Zero => Self::Top,
                    Self::Value(_) => return BTreeSet::new(),
                    Self::Bang(ref arg) => Self::Quest(Box::new(Self::Dual(arg.clone()))),
                    Self::Quest(ref arg) => Self::Bang(Box::new(Self::Dual(arg.clone()))),
                    Self::Dual(ref arg) => arg.as_ref().clone(),
                    Self::Times(ref lhs, ref rhs) => Self::Par(
                        Box::new(Self::Dual(lhs.clone())),
                        Box::new(Self::Dual(rhs.clone())),
                    ),
                    Self::Par(ref lhs, ref rhs) => Self::Times(
                        Box::new(Self::Dual(lhs.clone())),
                        Box::new(Self::Dual(rhs.clone())),
                    ),
                    Self::With(ref lhs, ref rhs) => Self::Plus(
                        Box::new(Self::Dual(lhs.clone())),
                        Box::new(Self::Dual(rhs.clone())),
                    ),
                    Self::Plus(ref lhs, ref rhs) => Self::With(
                        Box::new(Self::Dual(lhs.clone())),
                        Box::new(Self::Dual(rhs.clone())),
                    ),
                }])])])
            }
            Self::Times(ref blhs, ref brhs) => {
                let lhs = blhs.as_ref();
                let rhs = brhs.as_ref();
                let power_of_2 = 1_usize
                    .checked_shl(context.len().try_into().expect("Ridiculously huge value"))
                    .expect("More elements in a sequent than bits in a `usize`");
                (0..power_of_2)
                    .flat_map(|bits| {
                        let (mut lctx, mut rctx) = (Multiset::new(), Multiset::new());
                        for (i, ast) in context.rhs().iter().enumerate() {
                            let _ = if bits & (1 << i) == 0 {
                                &mut lctx
                            } else {
                                &mut rctx
                            }
                            .insert(ast.clone());
                        }
                        [
                            below.require([
                                RhsOnlyWithExchange::new(lctx.with([lhs.clone()])),
                                RhsOnlyWithExchange::new(rctx.with([rhs.clone()])),
                            ]),
                            below.require([
                                RhsOnlyWithExchange::new(rctx.with([lhs.clone()])),
                                RhsOnlyWithExchange::new(lctx.with([rhs.clone()])),
                            ]),
                        ]
                    })
                    .collect()
            }
            Self::Par(ref lhs, ref rhs) => BTreeSet::from_iter([
                below.require([context.with([lhs.as_ref().clone(), rhs.as_ref().clone()])])
            ]),
            Self::With(ref lhs, ref rhs) => BTreeSet::from_iter([below.require([
                context.with([lhs.as_ref().clone()]),
                context.with([rhs.as_ref().clone()]),
            ])]),
            Self::Plus(ref lhs, ref rhs) => BTreeSet::from_iter([
                below.require([context.with([lhs.as_ref().clone()])]),
                below.require([context.with([rhs.as_ref().clone()])]),
            ]),
        }
    }
}

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for Ast {
    #[inline]
    #[allow(
        clippy::as_conversions,
        clippy::indexing_slicing,
        clippy::unwrap_used,
        trivial_casts
    )]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        g.choose(
            &[
                (|_| Self::One) as fn(usize) -> Self,
                |_| Self::Bottom,
                |_| Self::Top,
                |_| Self::Zero,
                |s| Self::Value(usize::arbitrary(&mut quickcheck::Gen::new(s))),
                |s| {
                    Self::Bang(Box::arbitrary(&mut quickcheck::Gen::new(
                        s.saturating_sub(1),
                    )))
                },
                |s| {
                    Self::Quest(Box::arbitrary(&mut quickcheck::Gen::new(
                        s.saturating_sub(1),
                    )))
                },
                |s| {
                    Self::Dual(Box::arbitrary(&mut quickcheck::Gen::new(
                        s.saturating_sub(1),
                    )))
                },
                |s| {
                    let mut r = quickcheck::Gen::new(s.saturating_sub(1).overflowing_shr(1).0);
                    Self::Times(Box::arbitrary(&mut r), Box::arbitrary(&mut r))
                },
                |s| {
                    let mut r = quickcheck::Gen::new(s.saturating_sub(1).overflowing_shr(1).0);
                    Self::Par(Box::arbitrary(&mut r), Box::arbitrary(&mut r))
                },
                |s| {
                    let mut r = quickcheck::Gen::new(s.saturating_sub(1).overflowing_shr(1).0);
                    Self::With(Box::arbitrary(&mut r), Box::arbitrary(&mut r))
                },
                |s| {
                    let mut r = quickcheck::Gen::new(s.saturating_sub(1).overflowing_shr(1).0);
                    Self::Plus(Box::arbitrary(&mut r), Box::arbitrary(&mut r))
                },
            ][..g.size().clamp(4, 12)],
        )
        .unwrap()(g.size())
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match *self {
            Self::One => Box::new(core::iter::empty()),
            Self::Bottom => Box::new(core::iter::once(Self::One)),
            Self::Top => Box::new([Self::One, Self::Bottom].into_iter()),
            Self::Zero => Box::new([Self::One, Self::Bottom, Self::Top].into_iter()),
            Self::Value(i) => Box::new(
                [Self::One, Self::Bottom, Self::Top, Self::Zero]
                    .into_iter()
                    .chain(i.shrink().map(Self::Value)),
            ),
            Self::Bang(ref arg) => Box::new(
                Self::Value(usize::MAX)
                    .shrink()
                    .chain(arg.as_ref().shrink())
                    .chain(arg.shrink().map(Self::Bang)),
            ),
            Self::Quest(ref arg) => Box::new(
                Self::Bang(arg.clone())
                    .shrink()
                    .chain(arg.shrink().map(Self::Quest)),
            ),
            Self::Dual(ref arg) => Box::new(
                Self::Quest(arg.clone())
                    .shrink()
                    .chain(arg.shrink().map(Self::Dual)),
            ),
            Self::Times(ref lhs, ref rhs) => Box::new(
                Self::Quest(lhs.clone())
                    .shrink()
                    .chain(Self::Quest(rhs.clone()).shrink())
                    .chain(
                        (lhs.clone(), rhs.clone())
                            .shrink()
                            .map(|(tl, tr)| Self::Times(tl, tr)),
                    ),
            ),
            Self::Par(ref lhs, ref rhs) => Box::new(
                Self::Times(lhs.clone(), rhs.clone()).shrink().chain(
                    (lhs.clone(), rhs.clone())
                        .shrink()
                        .map(|(tl, tr)| Self::Par(tl, tr)),
                ),
            ),
            Self::With(ref lhs, ref rhs) => Box::new(
                Self::Par(lhs.clone(), rhs.clone()).shrink().chain(
                    (lhs.clone(), rhs.clone())
                        .shrink()
                        .map(|(tl, tr)| Self::With(tl, tr)),
                ),
            ),
            Self::Plus(ref lhs, ref rhs) => Box::new(
                Self::With(lhs.clone(), rhs.clone()).shrink().chain(
                    (lhs.clone(), rhs.clone())
                        .shrink()
                        .map(|(tl, tr)| Self::Plus(tl, tr)),
                ),
            ),
        }
    }
}

#[test]
fn cant_prove_0() {
    assert_eq!(prove(Ast::Zero), Err(Error::RanOutOfPaths));
}

#[test]
fn prove_1() {
    assert_eq!(prove(Ast::One), Ok(()));
}

#[test]
fn prove_top() {
    assert_eq!(prove(Ast::Top), Ok(()));
}

#[test]
fn prove_zero_par_top() {
    assert_eq!(prove(Ast::Zero.par(Ast::Top)), Ok(()));
}

#[test]
fn prove_0_implies_0() {
    assert_eq!(prove(Ast::Zero - Ast::Zero), Ok(()));
}

#[test]
fn prove_0_plus_1() {
    assert_eq!(prove(Ast::Zero + Ast::One), Ok(()));
}

#[test]
fn prove_1_plus_0() {
    assert_eq!(prove(Ast::One + Ast::Zero), Ok(()));
}

#[test]
fn prove_1_with_1() {
    assert_eq!(prove(Ast::One & Ast::One), Ok(()));
}

#[test]
fn prove_1_with_1_with_1() {
    assert_eq!(prove(Ast::One & Ast::One & Ast::One), Ok(()));
}

#[test]
fn prove_1_with_1_with_1_with_1() {
    assert_eq!(prove(Ast::One & Ast::One & Ast::One & Ast::One), Ok(()));
}

#[test]
fn prove_1_with_1_with_1_with_1_with_1() {
    assert_eq!(
        prove(Ast::One & Ast::One & Ast::One & Ast::One & Ast::One),
        Ok(())
    );
}

#[test]
fn cant_prove_0_with_1() {
    assert_eq!(prove(Ast::Zero & Ast::One), Err(Error::RanOutOfPaths),);
}

#[test]
fn cant_prove_1_with_0() {
    assert_eq!(prove(Ast::One & Ast::Zero), Err(Error::RanOutOfPaths),);
}

#[test]
fn a_with_b_implies_a() {
    assert_eq!(
        prove((Ast::Value(0) & Ast::Value(1)) - Ast::Value(0)),
        Ok(()),
    );
}

#[test]
fn a_with_b_implies_b() {
    assert_eq!(
        prove((Ast::Value(0) & Ast::Value(1)) - Ast::Value(1)),
        Ok(()),
    );
}

#[test]
fn bottom_implies_bottom() {
    assert_eq!(prove(Ast::Bottom - Ast::Bottom), Ok(()));
}

#[test]
fn prove_1_times_1() {
    assert_eq!(prove(Ast::One * Ast::One), Ok(()));
}

#[test]
fn cant_prove_1_times_0() {
    assert_eq!(prove(Ast::One * Ast::Zero), Err(Error::RanOutOfPaths));
}

#[test]
fn cant_prove_0_times_1() {
    assert_eq!(prove(Ast::Zero * Ast::One), Err(Error::RanOutOfPaths));
}

#[test]
fn cant_prove_0_times_0() {
    assert_eq!(prove(Ast::Zero * Ast::Zero), Err(Error::RanOutOfPaths));
}

#[test]
fn prove_1_times_1_times_1() {
    assert_eq!(prove(Ast::One * Ast::One * Ast::One), Ok(()));
}

#[test]
fn prove_1_times_1_times_1_times_1() {
    assert_eq!(prove(Ast::One * Ast::One * Ast::One * Ast::One), Ok(()));
}

#[test]
fn prove_1_times_1_times_1_times_1_times_1() {
    assert_eq!(
        prove(Ast::One * Ast::One * Ast::One * Ast::One * Ast::One),
        Ok(())
    );
}

#[test]
fn prove_1_implies_1_implies_1_implies_1_implies_1_times_1() {
    assert_eq!(
        prove(Ast::One - (Ast::One - (Ast::One - (Ast::One - (Ast::One * Ast::One))))),
        Ok(())
    );
}

#[test]
fn prove_excluded_middle_par() {
    assert_eq!(prove(Ast::Value(0).par(-Ast::Value(0))), Ok(()));
}

#[test]
fn cant_prove_excluded_middle_plus() {
    assert_eq!(
        prove(Ast::Value(0) + -Ast::Value(0)),
        Err(Error::RanOutOfPaths)
    );
}

#[test]
fn cant_prove_excluded_middle_with() {
    assert_eq!(
        prove(Ast::Value(0) & -Ast::Value(0)),
        Err(Error::RanOutOfPaths)
    );
}
