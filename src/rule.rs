/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A set of sequents above an inference line.

use crate::{Multiset, Sequent};
use core::hash::Hash;

/// A set of sequents above an inference line.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Rule<S: Sequent> {
    /// Name of the rule that allowed this inference.
    pub name: &'static str,
    /// Everything above the inference line: effectively next steps.
    pub above: Multiset<S>,
}

impl<S: Sequent> PartialEq for Rule<S> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.above == other.above
    }
}

impl<S: Sequent> Eq for Rule<S> {}

impl<S: Sequent> PartialOrd for Rule<S> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: Sequent> Ord for Rule<S> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.above.cmp(&other.above)
    }
}

impl<S: Sequent> Hash for Rule<S> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.above.hash(state);
    }
}

#[cfg(feature = "quickcheck")]
impl<S: Sequent + quickcheck::Arbitrary> quickcheck::Arbitrary for Rule<S> {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            name: "",
            above: quickcheck::Arbitrary::arbitrary(g),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.above
                .clone()
                .shrink()
                .map(|above| Self { name: "", above }),
        )
    }
}
