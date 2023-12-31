/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Unordered collection of (potentially many of the same) elements.

use core::num::NonZeroUsize;
use std::collections::{btree_map::IntoIter, BTreeMap};

/// Unordered collection of (potentially many of the same) elements.
#[repr(transparent)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Multiset<T: Ord>(pub(crate) BTreeMap<T, NonZeroUsize>);

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

impl<T: core::fmt::Display + Ord> core::fmt::Display for Multiset<T> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{{ ")?;
        for (element, count) in self.iter_unique() {
            write!(f, "{count}x {element}")?;
        }
        write!(f, "}}")
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
    /// # Panics
    /// If we overflow a `usize` (many other things, including maybe your death, will happen first).
    #[inline]
    #[allow(unsafe_code)]
    pub fn insert(&mut self, element: T) -> NonZeroUsize {
        *self
            .0
            .entry(element)
            .and_modify(|i| *i = i.checked_add(1).expect("Ridiculously huge value"))
            // SAFETY: Always 1, which is nonzero.
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
    /// # Panics
    /// If we overflow a `usize` (many other things, including maybe your death, will happen first).
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
        self.iter_repeat().next()
    }

    /// If this collection has exactly one element, view it without taking it out.
    #[must_use]
    #[inline(always)]
    pub fn only(&self) -> Option<&T> {
        let mut iter = self.iter_repeat();
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
        let mut iter = self.iter_repeat();
        iter.next().and_then(|first| {
            iter.next()
                .and_then(|second| iter.next().is_none().then_some((first, second)))
        })
    }

    /// Iterate over elements without copying them, visiting duplicate elements only once.
    #[inline]
    pub fn iter_unique(&self) -> std::collections::btree_map::Iter<'_, T, NonZeroUsize> {
        self.0.iter()
    }

    /// Iterate over elements, visiting duplicate elements only once.
    #[inline]
    pub fn into_iter_unique(self) -> std::collections::btree_map::IntoKeys<T, NonZeroUsize> {
        self.0.into_keys()
    }

    /// Iterate over elements without copying them, visiting duplicate elements more than once.
    #[inline]
    pub fn iter_repeat(&self) -> impl Iterator<Item = &T> {
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

    /// Iterate over elements, visiting duplicate elements more than once.
    #[inline]
    pub fn into_iter_repeat(self) -> IntoIterRepeat<T> {
        self.0
            .into_iter()
            .flat_map(|(t, i)| core::iter::repeat(t).take(i.get()))
    }
}

/// Output of `Multiset::into_iter_repeat`.
type IntoIterRepeat<T> = core::iter::FlatMap<
    IntoIter<T, NonZeroUsize>,
    core::iter::Take<core::iter::Repeat<T>>,
    fn((T, NonZeroUsize)) -> core::iter::Take<core::iter::Repeat<T>>,
>;

impl<T: Clone + Ord> IntoIterator for Multiset<T> {
    type Item = T;
    type IntoIter = IntoIterRepeat<T>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_iter_repeat()
    }
}

#[cfg(feature = "quickcheck")]
impl<T: quickcheck::Arbitrary + Ord> quickcheck::Arbitrary for Multiset<T> {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self::from_iter(Vec::arbitrary(g))
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.iter_repeat()
                .cloned()
                .collect::<Vec<_>>()
                .shrink()
                .map(Self::from_iter),
        )
    }
}
