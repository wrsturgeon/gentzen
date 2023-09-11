/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A turnstile symbol with comma-separated expressions on either (but currently just one) side.

use crate::{Infer, Multiset, Sequent};
use core::{fmt::Display, hash::Hash};

/// A turnstile symbol with comma-separated expressions on either (but currently just one) side.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct RhsOnlyWithExchange<Item: Clone + Display + Hash + Infer<Self> + Ord> {
    // /// Left side of the turnstile, on which comma means times.
    // pub(crate) lhs: Multiset<Item>,
    /// Right side of the turnstile, on which comma means par.
    pub(crate) rhs: Multiset<Item>,
}

impl<Item: Clone + Display + Hash + Infer<Self> + Ord> PartialOrd for RhsOnlyWithExchange<Item> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Item: Clone + Display + Hash + Infer<Self> + Ord> Ord for RhsOnlyWithExchange<Item> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.rhs.cmp(&other.rhs)
    }
}

impl<Item: Clone + Display + Hash + Infer<Self> + Ord> Sequent for RhsOnlyWithExchange<Item> {
    type Item = Item;
    type Lhs = ();
    type Rhs = Multiset<Item>;
    #[inline(always)]
    fn from_rhs(rhs_element: Self::Item) -> Self {
        let mut rhs = Multiset::new();
        let _ = rhs.insert(rhs_element);
        Self { rhs }
    }
    #[inline(always)]
    fn lhs(&self) -> &Self::Lhs {
        &()
    }
    #[inline(always)]
    fn rhs(&self) -> &Self::Rhs {
        &self.rhs
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

impl<Item: Clone + Display + Hash + Infer<Self> + Ord> RhsOnlyWithExchange<Item> {
    /// New sequent with exactly this on the right-hand side.
    #[must_use]
    #[inline(always)]
    pub const fn new(rhs: Multiset<Item>) -> Self {
        Self { rhs }
    }
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
    pub fn with<I: IntoIterator<Item = Item>>(&self, additions: I) -> Self {
        Self {
            rhs: self.rhs.with(additions),
        }
    }

    /// If this collection has exactly one element, view it without taking it out.
    #[must_use]
    #[inline(always)]
    pub fn only(&self) -> Option<&Item> {
        self.rhs.only()
    }

    /// Take an element by decreasing its count if we can.
    #[inline(always)]
    pub fn take(&mut self, element: &Item) -> bool {
        self.rhs.take(element)
    }
}

impl<Item: Clone + Display + Hash + Infer<Self> + Ord> Display for RhsOnlyWithExchange<Item> {
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
impl<Item: Clone + Display + Hash + Infer<Self> + Ord + quickcheck::Arbitrary> quickcheck::Arbitrary
    for RhsOnlyWithExchange<Item>
{
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
