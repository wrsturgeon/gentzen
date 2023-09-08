/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A turnstile symbol with comma-separated expressions on either (but currently just one) side.

use crate::{Ast, Multiset};
use std::{collections::BTreeSet, rc::Rc};

/// A turnstile symbol with comma-separated expressions on either (but currently just one) side.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Turnstile {
    // /// Left side of the turnstile, on which comma means times.
    // pub(crate) lhs: Multiset<Ast>,
    /// Right side of the turnstile, on which comma means par.
    pub(crate) rhs: Multiset<Ast>,
}

impl PartialOrd for Turnstile {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Turnstile {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.len().cmp(&other.len()) {
            diff @ (core::cmp::Ordering::Less | core::cmp::Ordering::Greater) => diff,
            // core::cmp::Ordering::Equal => match self.lhs.cmp(&other.lhs) {
            //     diff @ (core::cmp::Ordering::Less | core::cmp::Ordering::Greater) => diff,
            core::cmp::Ordering::Equal => self.rhs.cmp(&other.rhs),
            // },
        }
    }
}

impl Turnstile {
    /// New turnstile from an expression that will go on its right-hand side.
    #[must_use]
    #[inline(always)]
    pub fn new(ast: Ast) -> Self {
        let mut rhs = Multiset::new();
        let _ = rhs.insert(ast);
        Self {
            // lhs: Multiset::new(),
            rhs,
        }
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

impl core::fmt::Display for Turnstile {
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

#[cfg(test)]
impl quickcheck::Arbitrary for Turnstile {
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

/// Either from thin air, the only sequent above an inference line, or one of two sequents above an inference line.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum FamilyTree {
    /// From thin air.
    #[default]
    Stork,
    /// One sequent above the inference line.
    Linear(Rc<Trace>),
    /// Two sequents above the inference line.
    Split(Rc<Trace>),
}

#[cfg(test)]
impl quickcheck::Arbitrary for FamilyTree {
    #[inline]
    #[allow(clippy::same_functions_in_if_condition)]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            Self::Linear(Rc::new(Trace::arbitrary(g)))
        } else if bool::arbitrary(g) {
            Self::Split(Rc::new(Trace::arbitrary(g)))
        } else {
            Self::Stork
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            &Self::Stork => Box::new(core::iter::empty()),
            &Self::Linear(ref rc) => {
                Box::new(rc.shrink().map(|trace| Self::Linear(Rc::new(trace))))
            }
            &Self::Split(ref rc) => Box::new(
                Self::Linear(Rc::clone(rc))
                    .shrink()
                    .chain(rc.shrink().map(|trace| Self::Split(Rc::new(trace)))),
            ),
        }
    }
}

/// Turnstile together with its (linear) history.
#[derive(Clone, Debug)]
pub struct Trace {
    /// Current turnstile.
    pub(crate) current: Turnstile,
    /// All previous ones that led up to this one.
    pub(crate) history: FamilyTree,
}

impl PartialEq for Trace {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.current.eq(&other.current)
    }
}

impl Eq for Trace {}

impl PartialOrd for Trace {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Trace {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.current.cmp(&other.current) {
            diff @ (core::cmp::Ordering::Less | core::cmp::Ordering::Greater) => diff,
            core::cmp::Ordering::Equal => self.age().cmp(&other.age()),
        }
    }
}

impl core::hash::Hash for Trace {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.current.hash(state);
        // ignore history
    }
}

impl Trace {
    /// New trace with no history.
    #[inline(always)]
    pub fn from_thin_air(ast: Ast) -> Self {
        Self {
            current: Turnstile::new(ast),
            history: FamilyTree::Stork,
        }
    }

    /// Number of traced turnstiles before this one.
    #[inline(always)]
    pub fn age(&self) -> usize {
        let mut ancestor = &self.history;
        let mut acc: usize = 0;
        while let &(FamilyTree::Linear(ref parent) | FamilyTree::Split(ref parent)) = ancestor {
            acc = acc.checked_add(1).expect("Ridiculously huge value");
            ancestor = &parent.history;
        }
        acc
    }

    /// Continue with a single turnstile above the inference line.
    #[inline(always)]
    pub fn one(self: &Rc<Self>, child: Multiset<Ast>) -> BTreeSet<Self> {
        let mut bts = BTreeSet::new();
        let _ = bts.insert(Self {
            current: Turnstile { rhs: child },
            history: FamilyTree::Linear(Rc::clone(self)),
        });
        bts
    }

    /// Continue with two children.
    #[inline(always)]
    pub fn two(self: &Rc<Self>, lhs: Multiset<Ast>, rhs: Multiset<Ast>) -> BTreeSet<Self> {
        let mut bts = BTreeSet::new();
        let _ = bts.insert(Self {
            current: Turnstile { rhs: lhs },
            history: FamilyTree::Split(Rc::clone(self)),
        });
        let _ = bts.insert(Self {
            current: Turnstile { rhs },
            history: FamilyTree::Split(Rc::clone(self)),
        });
        bts
    }

    /// Clone and insert an element into the clone.
    #[must_use]
    #[inline(always)]
    pub fn with<I: IntoIterator<Item = Ast>>(&self, additions: I) -> Self {
        Self {
            current: self.current.with(additions),
            history: self.history.clone(),
        }
    }

    /// If this collection has exactly one element, view it without taking it out.
    #[must_use]
    #[inline(always)]
    pub fn only(&self) -> Option<&Ast> {
        self.current.only()
    }

    /// Take an element by decreasing its count if we can.
    #[inline(always)]
    pub fn take(&mut self, element: &Ast) -> bool {
        self.current.take(element)
    }
}

impl core::fmt::Display for Trace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.current, f)
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Trace {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            current: quickcheck::Arbitrary::arbitrary(g),
            history: quickcheck::Arbitrary::arbitrary(g),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.current.clone(), self.history.clone())
                .shrink()
                .map(|(current, history)| Self { current, history }),
        )
    }
}

/// Set of turnstiles.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Turnstiles(pub(crate) BTreeSet<Trace>);

impl PartialOrd for Turnstiles {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Turnstiles {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.len().cmp(&other.len()) {
            diff @ (core::cmp::Ordering::Less | core::cmp::Ordering::Greater) => diff,
            core::cmp::Ordering::Equal => self.0.cmp(&other.0),
        }
    }
}

impl Turnstiles {
    /// New set with no turnstiles (i.e. a rule with nothing above the inference line).
    #[must_use]
    #[inline(always)]
    pub const fn qed() -> Self {
        Self(BTreeSet::new())
    }

    /// New set from a single turnstile.
    #[must_use]
    #[inline(always)]
    pub fn new(singleton: Trace) -> Self {
        let mut bts = BTreeSet::new();
        let _ = bts.insert(singleton);
        Self(bts)
    }

    /// Total number of comma-separated expressions.
    #[must_use]
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether this set of sequents has been proven.
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// If this element has only one element, return it.
    /// Otherwise, return whether it's empty.
    /// # Errors
    /// Any size other than exactly 1.
    #[inline(always)]
    pub fn only(mut self) -> Result<Trace, bool> {
        match self.0.pop_first() {
            None => Err(true),
            Some(first) => {
                if self.0.pop_first().is_none() {
                    Ok(first)
                } else {
                    Err(false)
                }
            }
        }
    }
}

impl FromIterator<Trace> for Turnstiles {
    #[inline(always)]
    fn from_iter<T: IntoIterator<Item = Trace>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

impl<'a> IntoIterator for &'a Turnstiles {
    type Item = &'a Trace;
    type IntoIter = std::collections::btree_set::Iter<'a, Trace>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl core::fmt::Display for Turnstiles {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut iter = self.0.iter();
        if let Some(first) = iter.next() {
            write!(f, "{first}")?;
            for next in iter {
                write!(f, "   {next}")?;
            }
        }
        Ok(())
    }
}
