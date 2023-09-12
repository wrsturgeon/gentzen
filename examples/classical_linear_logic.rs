/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Abstract syntax tree for linear logic with sequent-calculus proof search built in.

#![deny(warnings)]
#![allow(clippy::needless_borrowed_reference)]

use gentzen::{sequents::RhsOnlyWithExchange, Infer, Multiset, Rule};

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

impl Infer<RhsOnlyWithExchange<Self>> for Ast {
    #[inline]
    fn above(&self, context: RhsOnlyWithExchange<Self>) -> Vec<Rule<RhsOnlyWithExchange<Self>>> {
        if context.rhs.contains(&Ast::Top)
            || context.rhs.iter().eq([&Self::Dual(Box::new(self.clone()))])
        {
            return vec![Rule {
                name: "axiom",
                above: [].into_iter().collect(),
            }];
        }
        match *self {
            Self::Top => vec![Rule {
                name: "\u{22a4}",
                above: [].into_iter().collect(),
            }],
            Self::One if context.is_empty() => vec![Rule {
                name: "1",
                above: [].into_iter().collect(),
            }],
            Self::Bang(ref arg) if matches!(context.only(), Some(&Self::Quest(_))) => {
                vec![Rule {
                    name: "!",
                    above: [context.with([arg.as_ref().clone()])].into_iter().collect(),
                }]
            }
            Self::One | Self::Zero | Self::Value(_) | Self::Bang(_) => vec![],
            Self::Bottom => vec![Rule {
                name: "",
                above: [context].into_iter().collect(),
            }],
            Self::Quest(ref arg) => vec![
                Rule {
                    name: "",
                    above: [context.clone()].into_iter().collect(),
                },
                Rule {
                    name: "",
                    above: [context.with([arg.as_ref().clone()])].into_iter().collect(),
                },
                Rule {
                    name: "",
                    above: [context.with([Self::Quest(arg.clone()), Self::Quest(arg.clone())])]
                        .into_iter()
                        .collect(),
                },
            ],
            Self::Dual(ref dual) => {
                vec![Rule {
                    name: "DeMorgan",
                    above: [context.with([match **dual {
                        Self::One => Self::Bottom,
                        Self::Bottom => Self::One,
                        Self::Top => Self::Zero,
                        Self::Zero => Self::Top,
                        Self::Value(_) => return vec![],
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
                    }])]
                    .into_iter()
                    .collect(),
                }]
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
                        for (i, ast) in context.rhs.iter().enumerate() {
                            let _ = if bits & (1 << i) == 0 {
                                &mut lctx
                            } else {
                                &mut rctx
                            }
                            .insert(ast.clone());
                        }
                        [
                            Rule {
                                name: "\u{2297}",
                                above: [
                                    RhsOnlyWithExchange::new(lctx.with([lhs.clone()])),
                                    RhsOnlyWithExchange::new(rctx.with([rhs.clone()])),
                                ]
                                .into_iter()
                                .collect(),
                            },
                            Rule {
                                name: "\u{2297}",
                                above: [
                                    RhsOnlyWithExchange::new(rctx.with([lhs.clone()])),
                                    RhsOnlyWithExchange::new(lctx.with([rhs.clone()])),
                                ]
                                .into_iter()
                                .collect(),
                            },
                        ]
                    })
                    .collect()
            }
            Self::Par(ref lhs, ref rhs) => {
                vec![Rule {
                    name: "\u{214b}",
                    above: [context.with([lhs.as_ref().clone(), rhs.as_ref().clone()])]
                        .into_iter()
                        .collect(),
                }]
            }
            Self::With(ref lhs, ref rhs) => vec![Rule {
                name: "&",
                above: [
                    context.with([lhs.as_ref().clone()]),
                    context.with([rhs.as_ref().clone()]),
                ]
                .into_iter()
                .collect(),
            }],
            Self::Plus(ref lhs, ref rhs) => vec![
                Rule {
                    name: "+L",
                    above: [context.with([lhs.as_ref().clone()])].into_iter().collect(),
                },
                Rule {
                    name: "+R",
                    above: [context.with([rhs.as_ref().clone()])].into_iter().collect(),
                },
            ],
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
    let original = Ast::Zero;
    prove(original).unwrap_err();
}

#[test]
fn prove_1() {
    let original = Ast::One;
    prove(original).unwrap();
}

#[test]
fn prove_top() {
    let original = Ast::Top;
    prove(original).unwrap();
}

#[test]
fn prove_zero_par_top() {
    let original = Ast::Zero.par(Ast::Top);
    prove(original).unwrap();
}

#[test]
fn prove_0_implies_0() {
    let original = Ast::Zero - Ast::Zero;
    prove(original).unwrap();
}

#[test]
fn prove_0_plus_1() {
    let original = Ast::Zero + Ast::One;
    prove(original).unwrap();
}

#[test]
fn prove_1_plus_0() {
    let original = Ast::One + Ast::Zero;
    prove(original).unwrap();
}

#[test]
fn prove_1_with_1() {
    let original = Ast::One & Ast::One;
    prove(original).unwrap();
}

#[test]
fn prove_1_with_1_with_1() {
    let original = Ast::One & Ast::One & Ast::One;
    prove(original).unwrap();
}

#[test]
fn prove_1_with_1_with_1_with_1() {
    let original = Ast::One & Ast::One & Ast::One & Ast::One;
    prove(original).unwrap();
}

#[test]
fn prove_1_with_1_with_1_with_1_with_1() {
    let original = Ast::One & Ast::One & Ast::One & Ast::One & Ast::One;
    prove(original).unwrap();
}

#[test]
fn cant_prove_0_with_1() {
    let original = Ast::Zero & Ast::One;
    prove(original).unwrap_err();
}

#[test]
fn cant_prove_1_with_0() {
    let original = Ast::One & Ast::Zero;
    prove(original).unwrap_err();
}

#[test]
fn a_with_b_implies_a() {
    let original = (Ast::Value(0) & Ast::Value(1)) - Ast::Value(0);
    prove(original).unwrap();
}

#[test]
fn a_with_b_implies_b() {
    let original = (Ast::Value(0) & Ast::Value(1)) - Ast::Value(1);
    prove(original).unwrap();
}

#[test]
fn bottom_implies_bottom() {
    let original = Ast::Bottom - Ast::Bottom;
    prove(original).unwrap();
}

#[test]
fn prove_1_times_1() {
    let original = Ast::One * Ast::One;
    prove(original).unwrap();
}

#[test]
fn cant_prove_1_times_0() {
    let original = Ast::One * Ast::Zero;
    prove(original).unwrap_err();
}

#[test]
fn cant_prove_0_times_1() {
    let original = Ast::Zero * Ast::One;
    prove(original).unwrap_err();
}

#[test]
fn cant_prove_0_times_0() {
    let original = Ast::Zero * Ast::Zero;
    prove(original).unwrap_err();
}

#[test]
fn prove_1_times_1_times_1() {
    let original = Ast::One * Ast::One * Ast::One;
    prove(original).unwrap();
}

#[test]
fn prove_1_times_1_times_1_times_1() {
    let original = Ast::One * Ast::One * Ast::One * Ast::One;
    prove(original).unwrap();
}

#[test]
fn prove_1_times_1_times_1_times_1_times_1() {
    let original = Ast::One * Ast::One * Ast::One * Ast::One * Ast::One;
    prove(original).unwrap();
}

#[test]
fn prove_1_implies_1_implies_1_implies_1_implies_1_times_1() {
    let original = Ast::One - (Ast::One - (Ast::One - (Ast::One - (Ast::One * Ast::One))));
    prove(original).unwrap();
}

#[test]
fn prove_excluded_middle_par() {
    let original = Ast::Value(0).par(-Ast::Value(0));
    prove(original).unwrap();
}

#[test]
fn cant_prove_excluded_middle_plus() {
    let original = Ast::Value(0) + -Ast::Value(0);
    assert_eq!(prove(original.clone()), Err(Error::RanOutOfPaths));
}

#[test]
fn cant_prove_excluded_middle_with() {
    let original = Ast::Value(0) & -Ast::Value(0);
    assert_eq!(prove(original.clone()), Err(Error::RanOutOfPaths));
}
