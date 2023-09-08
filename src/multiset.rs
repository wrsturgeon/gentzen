/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Unordered collection of (potentially many of the same) elements.

use core::num::NonZeroUsize;
use std::{collections::BTreeMap, rc::Rc};

use crate::{
    turnstile::{FamilyTree, Trace},
    Turnstile,
};

/// Unordered collection of (potentially many of the same) elements.
#[repr(transparent)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Multiset<T: Ord>(BTreeMap<T, NonZeroUsize>);

impl<T: Ord> Default for Multiset<T> {
    #[inline]
    fn default() -> Self {
        #[allow(clippy::default_trait_access)]
        Self(Default::default())
    }
}

impl<T: Ord> PartialOrd for Multiset<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord> Ord for Multiset<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.len().cmp(&other.len()) {
            diff @ (core::cmp::Ordering::Less | core::cmp::Ordering::Greater) => diff,
            core::cmp::Ordering::Equal => self.0.cmp(&other.0),
        }
    }
}

impl<T: Ord> Multiset<T> {
    /// Empty multiset.
    #[must_use]
    #[inline(always)]
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Add an element to the set, even if it's a duplicate. Return how many there _now_ are.
    #[inline]
    #[allow(unsafe_code)]
    pub fn insert(&mut self, element: T) -> NonZeroUsize {
        *self
            .0
            .entry(element)
            .and_modify(|i| *i = i.checked_add(1).expect("Ridiculously huge value"))
            // SAFETY:
            // Always 1, which is nonzero.
            .or_insert(unsafe { NonZeroUsize::new_unchecked(1) })
    }

    /// Look for an element, no matter how many, without changing anything.
    #[inline(always)]
    pub fn contains(&self, element: &T) -> bool {
        self.0.contains_key(element)
    }

    /// Take an element by decreasing its count if we can.
    #[inline]
    pub fn take(&mut self, element: &T) -> bool {
        match self.0.get_mut(element) {
            Some(i) => {
                if let Some(decr) = NonZeroUsize::new(i.get().overflowing_sub(1).0) {
                    *i = decr;
                    return true;
                }
            }
            None => return false,
        }
        let _ = self.0.remove(element);
        true
    }

    /// Whole number of elements, counting all duplicates.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.values().fold(0, |acc, i| {
            acc.checked_add(i.get()).expect("Ridiculously huge value")
        })
    }

    /// View an arbitrary element without taking it out.
    #[must_use]
    #[inline(always)]
    pub fn any_element(&self) -> Option<&T> {
        self.iter().next()
    }

    /// If this collection has exactly one element, view it without taking it out.
    #[must_use]
    #[inline(always)]
    pub fn only(&self) -> Option<&T> {
        let mut iter = self.iter();
        let maybe = iter.next();
        if iter.next().is_none() {
            maybe
        } else {
            None
        }
    }

    /// If this collection has exactly two elements, view them without taking them out.
    #[must_use]
    #[inline(always)]
    pub fn pair(&self) -> Option<(&T, &T)> {
        let mut iter = self.iter();
        iter.next().and_then(|first| {
            iter.next()
                .and_then(|second| iter.next().is_none().then_some((first, second)))
        })
    }

    /// Iterate over elements without copying them, visiting duplicate elements only once.
    #[inline]
    pub fn iter_unique(&self) -> impl Iterator<Item = (&T, &NonZeroUsize)> {
        self.0.iter()
    }

    /// Iterate over elements without copying them, visiting duplicate elements more than once.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0
            .iter()
            .flat_map(|(t, i)| core::iter::repeat(t).take(i.get()))
    }

    /// Whether there are any elements.
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Continue with a single turnstile above the inference line.
    #[must_use]
    #[inline(always)]
    pub fn and_then(child: Turnstile, parent: Rc<Trace>) -> Trace {
        Trace {
            current: child,
            history: FamilyTree::Linear(parent),
        }
    }

    /// Continue with two children.
    #[must_use]
    #[inline(always)]
    pub fn split(lhs: Turnstile, rhs: Turnstile, parent: Rc<Trace>) -> (Trace, Trace) {
        (
            Trace {
                current: lhs,
                history: FamilyTree::Split(Rc::clone(&parent)),
            },
            Trace {
                current: rhs,
                history: FamilyTree::Split(parent),
            },
        )
    }
}

impl<T: Clone + Ord> Multiset<T> {
    /// Clone and insert an element into the clone.
    #[inline]
    #[must_use]
    pub fn with<I: IntoIterator<Item = T>>(&self, additions: I) -> Self {
        let mut ms = self.clone();
        for element in additions {
            let _ = ms.insert(element);
        }
        ms
    }
}

impl<T: Ord> FromIterator<T> for Multiset<T> {
    #[inline(always)]
    #[allow(unsafe_code)]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut ms = Self::new();
        for element in iter {
            let _ = ms.insert(element);
        }
        ms
    }
}

#[cfg(test)]
impl<T: quickcheck::Arbitrary + Ord> quickcheck::Arbitrary for Multiset<T> {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(quickcheck::Arbitrary::arbitrary(g))
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(Self))
    }
}
