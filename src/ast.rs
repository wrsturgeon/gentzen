/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Abstract syntax tree for linear logic with sequent-calculus proof search built in.

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

/// Ordering on infix operators (based on operator precedence).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Infix {
    /// Multiplicative conjunction.
    Times,
    /// Multiplicative disjunction.
    Par,
    /// Additive conjunction.
    With,
    /// Additive disjunction.
    Plus,
}

impl Infix {
    /// Given arguments, turn this into an AST node.
    pub(crate) const fn into_ast(self, lhs: Box<Ast>, rhs: Box<Ast>) -> Ast {
        match self {
            Self::Times => Ast::Times(lhs, rhs),
            Self::Par => Ast::Par(lhs, rhs),
            Self::With => Ast::With(lhs, rhs),
            Self::Plus => Ast::Plus(lhs, rhs),
        }
    }
}

impl core::fmt::Display for Infix {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &Self::Times => '\u{2297}',
                &Self::Par => '\u{214b}',
                &Self::With => '&',
                &Self::Plus => '\u{2295}',
            },
        )
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Infix {
    #[inline]
    #[allow(clippy::unwrap_used)]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        *g.choose(&[Self::Times, Self::Par, Self::With, Self::Plus])
            .unwrap()
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            &Self::Times => Box::new(core::iter::empty()),
            &Self::Par => Box::new(core::iter::once(Self::Times)),
            &Self::With => Box::new([Self::Times, Self::Par].into_iter()),
            &Self::Plus => Box::new([Self::Times, Self::Par, Self::With].into_iter()),
        }
    }
}

impl core::fmt::Display for Ast {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            &Self::One => write!(f, "1"),
            &Self::Bottom => write!(f, "\u{22a5}"),
            &Self::Top => write!(f, "\u{22a4}"),
            &Self::Zero => write!(f, "0"),
            &Self::Value(i) => write!(f, "P{i}"),
            &Self::Bang(ref arg) => write!(f, "!({arg})"),
            &Self::Quest(ref arg) => write!(f, "?({arg})"),
            &Self::Dual(ref arg) => write!(f, "~({arg})"),
            &Self::Times(ref lhs, ref rhs) => write!(f, "({lhs}) \u{2297} ({rhs})"),
            &Self::Par(ref lhs, ref rhs) => {
                write!(f, "({lhs}) \u{214b} ({rhs})")
            }
            &Self::With(ref lhs, ref rhs) => write!(f, "({lhs}) & ({rhs})"),
            &Self::Plus(ref lhs, ref rhs) => write!(f, "({lhs}) \u{2295} ({rhs})"),
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

    /// Attempt to prove this expression with sequent-calculus proof search.
    /// # Errors
    /// If we can't.
    #[inline(always)]
    pub fn prove(self) -> Result<(), crate::proof::Error> {
        crate::proof::prove(self)
    }

    /// Infix operator, if there is one.
    #[inline]
    #[must_use]
    pub(crate) const fn infix_op(&self) -> Option<Infix> {
        match self {
            &(Self::One
            | Self::Bottom
            | Self::Top
            | Self::Zero
            | Self::Value(_)
            | Self::Dual(_)
            | Self::Bang(_)
            | Self::Quest(_)) => None,
            &Self::Times(_, _) => Some(Infix::Times),
            &Self::Par(_, _) => Some(Infix::Par),
            &Self::With(_, _) => Some(Infix::With),
            &Self::Plus(_, _) => Some(Infix::Plus),
        }
    }

    // /// Check if this is unambiguously structured.
    // /// # Errors
    // /// If it's not sorted (with an explanation).
    // #[inline]
    // pub fn sorted(&self) -> Result<(), String> {
    //     match self {
    //         &(Self::One
    //         | Self::Bottom
    //         | Self::Top
    //         | Self::Zero
    //         | Self::Value(_)
    //         | Self::Dual(_)) => Ok(()),
    //         &(Self::Bang(ref arg) | Self::Quest(ref arg)) => arg.sorted(),
    //         &(Self::Times(ref lhs, ref rhs)
    //         | Self::Par(ref lhs, ref rhs)
    //         | Self::With(ref lhs, ref rhs)
    //         | Self::Plus(ref lhs, ref rhs)) => {
    //             let self_op = self.infix_op();
    //             lhs.sorted()?;
    //             rhs.sorted()?;
    //             match (lhs.infix_op(), rhs.infix_op()) {
    //                 (None, None) => {
    //                     if lhs <= rhs {
    //                         Ok(())
    //                     } else {
    //                         Err(format!("[{lhs}] is not <= [{rhs}]"))
    //                     }
    //                 }
    //                 (Some(op_l), None) => {
    //                     if Some(op_l) <= self_op {
    //                         Ok(())
    //                     } else {
    //                         Err(format!(
    //                             "{op_l} is not <= {}",
    //                             self_op.map_or_else(|| "None".to_owned(), |op| op.to_string()),
    //                         ))
    //                     }
    //                 }
    //                 (None, Some(op)) => Err(format!(
    //                     "Operator ({op}) on the right side [{rhs}] but not on the left [{lhs}]",
    //                 )),
    //                 (Some(op_l), Some(op_r)) => {
    //                     if Some(op_l) <= self_op {
    //                         return Err(format!(
    //                             "{op_l} is not <= {}",
    //                             self_op.map_or_else(|| "None".to_owned(), |op| op.to_string()),
    //                         ));
    //                     }
    //                     if Some(op_r) < self_op {
    //                         return Err(format!(
    //                             "{op_r} is not < {}",
    //                             self_op.map_or_else(|| "None".to_owned(), |op| op.to_string()),
    //                         ));
    //                     }
    //                     if op_l <= op_r {
    //                         return Err(format!("{op_l} is not <= {op_r}",));
    //                     }
    //                     Ok(())
    //                 }
    //             }
    //         }
    //     }
    // }

    // /// Unambiguously order this expression.
    // #[inline]
    // #[must_use]
    // #[allow(unsafe_code)]
    // pub fn sort(self) -> Self {
    //     if let Some(op) = self.infix_op() {
    //         match self {
    //             Self::Times(lhs, rhs)
    //             | Self::Par(lhs, rhs)
    //             | Self::With(lhs, rhs)
    //             | Self::Plus(lhs, rhs) => {
    //                 let expl = Box::new(lhs.sort());
    //                 let expr = Box::new(rhs.sort());
    //                 match (expl.infix_op(), expr.infix_op()) {
    //                     (None, None) => { /* fall through */ }
    //                     (Some(_), None) => return op.into_ast(expl, expr),
    //                     (None, Some(_)) => return op.into_ast(expr, expl),
    //                     (Some(op_l), Some(op_r)) => match op_l.cmp(&op_r) {
    //                         core::cmp::Ordering::Less => return op.into_ast(expr, expl),
    //                         core::cmp::Ordering::Greater => return op.into_ast(expl, expr),
    //                         core::cmp::Ordering::Equal => { /* fall through */ }
    //                     },
    //                 }
    //                 if expr < expl {
    //                     op.into_ast(expr, expl)
    //                 } else {
    //                     op.into_ast(expl, expr)
    //                 }
    //             }
    //             Self::One
    //             | Self::Bottom
    //             | Self::Top
    //             | Self::Zero
    //             | Self::Value(_)
    //             | Self::Dual(_)
    //             | Self::Bang(_)
    //             | Self::Quest(_) => {
    //                 // SAFETY:
    //                 // Based on the output of `infix_op`.
    //                 unsafe { core::hint::unreachable_unchecked() }
    //             }
    //         }
    //     } else {
    //         match self {
    //             Self::One
    //             | Self::Bottom
    //             | Self::Top
    //             | Self::Zero
    //             | Self::Value(_)
    //             | Self::Dual(_) => self,
    //             Self::Bang(arg) => Self::Bang(Box::new(arg.sort())),
    //             Self::Quest(arg) => Self::Quest(Box::new(arg.sort())),
    //             Self::Times(..) | Self::Par(..) | Self::With(..) | Self::Plus(..) => {
    //                 // SAFETY:
    //                 // Based on the output of `infix_op`.
    //                 unsafe { core::hint::unreachable_unchecked() }
    //             }
    //         }
    //     }
    // }
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
        // match self {
        //     Self::One => Self::Bottom,
        //     Self::Bottom => Self::One,
        //     Self::Top => Self::Zero,
        //     Self::Zero => Self::Top,
        //     Self::Value(i) => Self::Dual(i),
        //     Self::Dual(i) => Self::Value(i),
        //     Self::Bang(arg) => Self::Quest(Box::new(-*arg)),
        //     Self::Quest(arg) => Self::Bang(Box::new(-*arg)),
        //     Self::Times(lhs, rhs) => Self::Par(Box::new(-*lhs), Box::new(-*rhs)),
        //     Self::Par(lhs, rhs) => Self::Times(Box::new(-*lhs), Box::new(-*rhs)),
        //     Self::With(lhs, rhs) => Self::Plus(Box::new(-*lhs), Box::new(-*rhs)),
        //     Self::Plus(lhs, rhs) => Self::With(Box::new(-*lhs), Box::new(-*rhs)),
        // }
        Self::Dual(Box::new(self))
    }
}

#[cfg(test)]
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
        match self {
            &Self::One => Box::new(core::iter::empty()),
            &Self::Bottom => Box::new(core::iter::once(Self::One)),
            &Self::Top => Box::new([Self::One, Self::Bottom].into_iter()),
            &Self::Zero => Box::new([Self::One, Self::Bottom, Self::Top].into_iter()),
            &Self::Value(i) => Box::new(
                [Self::One, Self::Bottom, Self::Top, Self::Zero]
                    .into_iter()
                    .chain(i.shrink().map(Self::Value)),
            ),
            &Self::Bang(ref arg) => Box::new(
                Self::Value(usize::MAX)
                    .shrink()
                    .chain(arg.as_ref().shrink())
                    .chain(arg.shrink().map(Self::Bang)),
            ),
            &Self::Quest(ref arg) => Box::new(
                Self::Bang(arg.clone())
                    .shrink()
                    .chain(arg.shrink().map(Self::Quest)),
            ),
            &Self::Dual(ref arg) => Box::new(
                Self::Quest(arg.clone())
                    .shrink()
                    .chain(arg.shrink().map(Self::Dual)),
            ),
            &Self::Times(ref lhs, ref rhs) => Box::new(
                Self::Quest(lhs.clone())
                    .shrink()
                    .chain(Self::Quest(rhs.clone()).shrink())
                    .chain(
                        (lhs.clone(), rhs.clone())
                            .shrink()
                            .map(|(tl, tr)| Self::Times(tl, tr)),
                    ),
            ),
            &Self::Par(ref lhs, ref rhs) => Box::new(
                Self::Times(lhs.clone(), rhs.clone()).shrink().chain(
                    (lhs.clone(), rhs.clone())
                        .shrink()
                        .map(|(tl, tr)| Self::Par(tl, tr)),
                ),
            ),
            &Self::With(ref lhs, ref rhs) => Box::new(
                Self::Par(lhs.clone(), rhs.clone()).shrink().chain(
                    (lhs.clone(), rhs.clone())
                        .shrink()
                        .map(|(tl, tr)| Self::With(tl, tr)),
                ),
            ),
            &Self::Plus(ref lhs, ref rhs) => Box::new(
                Self::With(lhs.clone(), rhs.clone()).shrink().chain(
                    (lhs.clone(), rhs.clone())
                        .shrink()
                        .map(|(tl, tr)| Self::Plus(tl, tr)),
                ),
            ),
        }
    }
}
