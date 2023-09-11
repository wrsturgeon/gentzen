/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A set of sequents above an inference line plus a reference to the sequent below the inference line.

use crate::{thunk::Thunk, Sequent};
use core::{fmt::Display, hash::Hash};
use std::{collections::BTreeSet, rc::Rc};

/// A set of sequents above an inference line plus a reference to the sequent below the inference line.
#[derive(Clone, Debug)]
pub struct Inference<S: Sequent> {
    /// Everything above the inference line: effectively next steps.
    pub(crate) above: BTreeSet<S>,
    /// If `self` is proven true/false,
    /// it would immediately follow that
    /// `self.history` is proven the same.
    pub(crate) below: Rc<S>,
}

impl<S: Sequent> PartialEq for Inference<S> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.above == other.above
    }
}

impl<S: Sequent> Eq for Inference<S> {}

impl<S: Sequent> PartialOrd for Inference<S> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: Sequent> Ord for Inference<S> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.above.cmp(&other.above) {
            diff @ (core::cmp::Ordering::Less | core::cmp::Ordering::Greater) => diff,
            core::cmp::Ordering::Equal => self.below.cmp(&other.below),
        }
    }
}

impl<S: Sequent> Hash for Inference<S> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.above.hash(state);
    }
}

impl<S: Sequent> Display for Inference<S> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} (would prove {})", self.without_history(), self.below,)
    }
}

impl<S: Sequent> Inference<S> {
    /// Print without what it would prove (i.e. history).
    #[inline]
    #[allow(clippy::arithmetic_side_effects)]
    pub(crate) fn without_history(&self) -> String {
        let mut iter = self.above.iter();
        iter.next().map_or_else(
            || "{ }".to_owned(),
            |first| {
                iter.fold(format!("{{ {first}"), |acc, sequent| {
                    acc + &format!("   {sequent}")
                }) + " }"
            },
        )
    }

    /// Check if we have proofs already cached for each sequent above the inference line.
    #[inline]
    pub(crate) fn proven(&self, thunk: &Thunk<S>) -> bool {
        self.above.iter().all(|sequent| thunk.proven(sequent))
    }
}

#[cfg(feature = "quickcheck")]
impl<S: Sequent + quickcheck::Arbitrary> quickcheck::Arbitrary for Inference<S> {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            above: quickcheck::Arbitrary::arbitrary(g),
            below: Rc::new(quickcheck::Arbitrary::arbitrary(g)),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.above.clone(), self.below.as_ref().clone())
                .shrink()
                .map(|(above, below)| Self {
                    above,
                    below: Rc::new(below),
                }),
        )
    }
}
