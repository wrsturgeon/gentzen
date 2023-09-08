/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A turnstile symbol with comma-separated expressions on either (but currently just one) side.

use crate::{Ast, Multiset};
use std::rc::Rc;

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

// /// Either from thin air, the only sequent above an inference line, or one of two sequents above an inference line.
// #[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
// pub(crate) enum FamilyTree {
//     /// From thin air.
//     #[default]
//     Stork,
//     /// One sequent above the inference line.
//     Linear(Rc<Trace>),
//     /// Two sequents above the inference line.
//     Split(Rc<Trace>),
// }

// #[cfg(test)]
// impl quickcheck::Arbitrary for FamilyTree {
//     #[inline]
//     #[allow(clippy::same_functions_in_if_condition)]
//     fn arbitrary(g: &mut quickcheck::Gen) -> Self {
//         if bool::arbitrary(g) {
//             Self::Linear(Rc::new(Trace::arbitrary(g)))
//         } else if bool::arbitrary(g) {
//             Self::Split(Rc::new(Trace::arbitrary(g)))
//         } else {
//             Self::Stork
//         }
//     }
//     #[inline]
//     fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
//         match self {
//             &Self::Stork => Box::new(core::iter::empty()),
//             &Self::Linear(ref rc) => {
//                 Box::new(rc.shrink().map(|trace| Self::Linear(Rc::new(trace))))
//             }
//             &Self::Split(ref rc) => Box::new(
//                 Self::Linear(Rc::clone(rc))
//                     .shrink()
//                     .chain(rc.shrink().map(|trace| Self::Split(Rc::new(trace)))),
//             ),
//         }
//     }
// }

/// Turnstile together with its (linear) history.
#[derive(Clone, Debug, Default)]
pub struct Trace {
    /// Current turnstile.
    pub(crate) current: Turnstile,
    /// All previous turnstiles that led up to this one.
    pub(crate) history: Option<Rc<Trace>>,
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
    pub const fn from_thin_air(current: Turnstile) -> Self {
        Self {
            current,
            history: None,
        }
    }

    /// Number of traced turnstiles before this one.
    #[inline(always)]
    pub fn age(&self) -> usize {
        let mut ancestor = &self.history;
        let mut acc: usize = 0;
        while let &Some(ref parent) = ancestor {
            acc = acc.checked_add(1).expect("Ridiculously huge value");
            ancestor = &parent.history;
        }
        acc
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

/// Convert to a linked list.
#[inline]
fn rc_list(v: Vec<Turnstile>) -> Option<Rc<Trace>> {
    v.into_iter().fold(None, |history, current| {
        Some(Rc::new(Trace { current, history }))
    })
}

/// Convert from a linked list.
#[inline]
fn from_rc_list(mut history: Option<&Rc<Trace>>) -> Vec<Turnstile> {
    let mut acc = vec![];
    while let Some(parent) = history {
        acc.push(parent.current.clone());
        history = parent.history.as_ref();
    }
    acc
}

#[cfg(test)]
impl quickcheck::Arbitrary for Trace {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            current: quickcheck::Arbitrary::arbitrary(g),
            history: rc_list(Vec::arbitrary(g)),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.current.clone(), from_rc_list(self.history.as_ref()))
                .shrink()
                .map(|(current, history)| Self {
                    current,
                    history: rc_list(history),
                }),
        )
    }
}

/// Set of turnstiles all together on top of an inference line.
#[derive(Clone, Debug, Default)]
pub struct Split {
    /// Left-hand turnstile.
    pub(crate) lhs: Trace,
    /// Right-hand turnstile.
    pub(crate) rhs: Trace,
    /// All previous turnstiles that led up to this one.
    pub(crate) history: Rc<Trace>,
}

impl PartialEq for Split {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (&self.lhs, &self.rhs) == (&other.lhs, &other.rhs)
            || (&self.rhs, &self.lhs) == (&other.lhs, &other.rhs)
    }
}

impl Eq for Split {}

impl PartialOrd for Split {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Split {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let (lmin, lmax) = self.sort();
        let (rmin, rmax) = other.sort();
        match lmin.cmp(rmin) {
            diff @ (core::cmp::Ordering::Less | core::cmp::Ordering::Greater) => diff,
            core::cmp::Ordering::Equal => lmax.cmp(rmax),
        }
    }
}

impl core::hash::Hash for Split {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        let (min, max) = self.sort();
        min.hash(state);
        max.hash(state);
    }
}

impl core::fmt::Display for Split {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (lhs, rhs) = self.sort();
        write!(f, "[ {lhs}   {rhs} ]")
    }
}

impl Split {
    /// Fix an arbitrary ordering.
    #[inline]
    #[must_use]
    pub fn sort(&self) -> (&Trace, &Trace) {
        if self.rhs < self.lhs {
            (&self.rhs, &self.lhs)
        } else {
            (&self.lhs, &self.rhs)
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Split {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let lhs = quickcheck::Arbitrary::arbitrary(g);
        let rhs = quickcheck::Arbitrary::arbitrary(g);
        loop {
            if let Some(history) = rc_list(Vec::arbitrary(g)) {
                return Self { lhs, rhs, history };
            }
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (
                self.lhs.clone(),
                self.rhs.clone(),
                from_rc_list(Some(&Rc::new(self.history.as_ref().clone()))),
            )
                .shrink()
                .filter_map(|(lhs, rhs, history)| {
                    Some(Self {
                        lhs,
                        rhs,
                        history: rc_list(history)?,
                    })
                }),
        )
    }
}
