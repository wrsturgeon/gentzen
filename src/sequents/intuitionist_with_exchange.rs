/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A turnstile symbol with a comma-separated expression on the left and a single expression on the right.

use crate::{Infer, Multiset, Sequent};
use core::{
    fmt::{Debug, Display},
    hash::Hash,
};

/// A turnstile symbol with a comma-separated expression on the left and a single expression on the right.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IntuitionistWithExchange<Item: Debug + Display + Hash + Infer<Self> + Ord> {
    /// Left side of the turnstile, on which comma means times.
    pub lhs: Multiset<Item>,
    /// Right side of the turnstile, on which comma means par.
    pub rhs: Item,
}

impl<Item: Debug + Display + Hash + Infer<Self> + Ord> Sequent for IntuitionistWithExchange<Item> {
    type Item = Item;
    #[inline(always)]
    fn from_rhs(rhs: Self::Item) -> Self {
        Self {
            lhs: Multiset::new(),
            rhs,
        }
    }
    #[inline]
    fn sample(&self) -> Vec<(Self::Item, Self)> {
        self.lhs
            .iter_unique()
            .map(|(ast, _)| {
                let mut ablation = self.lhs.clone();
                let _ = ablation.take(ast);
                (
                    ast.clone(),
                    Self {
                        lhs: ablation,
                        rhs: self.rhs.clone(),
                    },
                )
            })
            .collect()
    }
}

impl<Item: Debug + Display + Hash + Infer<Self> + Ord> IntuitionistWithExchange<Item> {
    /// New sequent with exactly this on the right-hand side.
    #[must_use]
    #[inline(always)]
    pub const fn new(lhs: Multiset<Item>, rhs: Item) -> Self {
        Self { lhs, rhs }
    }
    /// Total number of comma-separated expressions, not counting the right-hand side.
    #[must_use]
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.lhs.len()
    }

    /// Whether there are any statements on either side.
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.lhs.is_empty()
    }

    /// Clone and insert an element into the clone.
    #[must_use]
    #[inline(always)]
    pub fn with<I: IntoIterator<Item = Item>>(&self, additions: I) -> Self {
        Self {
            lhs: self.lhs.with(additions),
            rhs: self.rhs.clone(),
        }
    }
}

impl<Item: Debug + Display + Hash + Infer<Self> + Ord> Display for IntuitionistWithExchange<Item> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "\u{22a2}")?;
        let mut iter = self.lhs.iter();
        if let Some(first) = iter.next() {
            write!(f, "{first}")?;
            for next in iter {
                write!(f, ", {next}")?;
            }
            write!(f, " \u{22a2} {}", self.rhs)
        } else {
            write!(f, "\u{22a2} {}", self.rhs)
        }
    }
}

#[cfg(feature = "quickcheck")]
impl<Item: Debug + Display + Hash + Infer<Self> + Ord + quickcheck::Arbitrary> quickcheck::Arbitrary
    for IntuitionistWithExchange<Item>
{
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            lhs: quickcheck::Arbitrary::arbitrary(g),
            rhs: quickcheck::Arbitrary::arbitrary(g),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.lhs.clone(), self.rhs.clone())
                .shrink()
                .map(|(lhs, rhs)| Self { lhs, rhs }),
        )
    }
}
