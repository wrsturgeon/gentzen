/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A turnstile symbol with comma-separated expressions on either (but currently just one) side.

use crate::{Ast, Multiset, Sequent};

/// A turnstile symbol with comma-separated expressions on either (but currently just one) side.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct RhsOnlyWithExchange {
    // /// Left side of the turnstile, on which comma means times.
    // pub(crate) lhs: Multiset<Ast>,
    /// Right side of the turnstile, on which comma means par.
    pub(crate) rhs: Multiset<Ast>,
}

impl PartialOrd for RhsOnlyWithExchange {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RhsOnlyWithExchange {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.rhs.cmp(&other.rhs)
    }
}

impl Sequent for RhsOnlyWithExchange {
    type Item = Ast;
    type Lhs = ();
    type Rhs = Multiset<Ast>;
    #[inline(always)]
    fn from_rhs(rhs_element: Self::Item) -> Self {
        let mut rhs = Multiset::new();
        let _ = rhs.insert(rhs_element);
        Self { rhs }
    }
    #[inline(always)]
    fn lhs_contains(&self, _: &Self::Item) -> bool {
        false
    }
    #[inline(always)]
    fn rhs_contains(&self, element: &Self::Item) -> bool {
        self.rhs.contains(element)
    }
    #[inline]
    fn sample(&self) -> Vec<(Self::Item, Self)> {
        self.rhs
            .iter_unique()
            .map(|(ast, _)| {
                let mut ablation = self.rhs.clone();
                let _ = ablation.take(ast);
                (ast.clone(), Self { rhs: ablation })
            })
            .collect()
    }
}

impl RhsOnlyWithExchange {
    /// Total number of comma-separated expressions.
    #[must_use]
    #[inline(always)]
    pub fn len(&self) -> usize {
        // self.lhs.len() +
        self.rhs.len()
    }

    /// Whether there are any statements on either side.
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.rhs.is_empty()
    }

    /// Clone and insert an element into the clone.
    #[must_use]
    #[inline(always)]
    pub fn with<I: IntoIterator<Item = Ast>>(&self, additions: I) -> Self {
        Self {
            rhs: self.rhs.with(additions),
        }
    }

    /// If this collection has exactly one element, view it without taking it out.
    #[must_use]
    #[inline(always)]
    pub fn only(&self) -> Option<&Ast> {
        self.rhs.only()
    }

    /// Take an element by decreasing its count if we can.
    #[inline(always)]
    pub fn take(&mut self, element: &Ast) -> bool {
        self.rhs.take(element)
    }
}

impl core::fmt::Display for RhsOnlyWithExchange {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "\u{22a2}")?;
        let mut iter = self.rhs.iter();
        if let Some(first) = iter.next() {
            write!(f, " {first}")?;
            for next in iter {
                write!(f, ", {next}")?;
            }
        }
        Ok(())
    }
}

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for RhsOnlyWithExchange {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            // lhs: quickcheck::Arbitrary::arbitrary(g),
            rhs: quickcheck::Arbitrary::arbitrary(g),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (/* self.lhs, */self.rhs)
                .shrink()
                .map(|/* lhs, */ rhs| Self { /* lhs, */ rhs, }),
        )
    }
}
