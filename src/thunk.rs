/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Cache any finished results automatically.

use crate::Sequent;
use core::cmp::Reverse;
use std::collections::{hash_map::Entry, BinaryHeap, HashMap};

/// Cache any finished results automatically.
pub(crate) struct Thunk<S: Sequent> {
    /// Record of what we've seen and, within that set, what we've proven.
    cache: HashMap<S, bool>,
    /// Smallest-first queue of unproven sequents.
    queue: BinaryHeap<Reverse<S>>,
    /// The sequent we're trying to prove overall.
    original: S,
}

/// This specific sequent (not the whole proof) has already been proven.
pub(crate) struct AlreadyProven;
/// The entire proof is finished.
pub(crate) struct Qed;

impl<S: Sequent> Thunk<S> {
    /// Create a new queue with only this original expression.
    #[inline]
    pub(crate) fn new(expression: S::Item) -> Self {
        let sequent = S::from_rhs(expression);
        let mut q = Self {
            cache: HashMap::new(),
            queue: BinaryHeap::new(),
            original: sequent.clone(),
        };
        #[allow(unsafe_code)]
        // SAFETY: Empty above: can't have already been proven.
        unsafe {
            q.push(sequent).unwrap_unchecked();
        }
        q
    }

    /// Add a sequent to be proven, or if it's already been proven, return `Err(AlreadyProven)`.
    #[inline]
    pub(crate) fn push(&mut self, sequent: S) -> Result<(), AlreadyProven> {
        match self.cache.entry(sequent.clone()).or_insert(false) {
            &mut false => {
                dbg_println!("    Adding {sequent}");
                self.queue.push(Reverse(sequent));
                Ok(())
            }
            &mut true => {
                dbg_println!("    Already proved {sequent}");
                Err(AlreadyProven)
            }
        }
    }

    /// Mark a sequent proven.
    #[inline]
    #[cfg_attr(
        any(test, debug_assertions),
        allow(
            clippy::needless_pass_by_value,
            clippy::panic,
            clippy::panic_in_result_fn,
            unreachable_code,
            unused_mut,
            unused_variables
        )
    )]
    pub(crate) fn cache(&mut self, sequent: S) -> Result<(), Qed> {
        if sequent == self.original {
            Err(Qed)
        } else {
            match self.cache.entry(
                #[cfg(any(test, debug_assertions))]
                sequent.clone(),
                #[cfg(not(any(test, debug_assertions)))]
                sequent,
            ) {
                Entry::Vacant(empty) => {
                    #[cfg(any(test, debug_assertions))]
                    panic!(
                        "Tried to mark {sequent} proven, \
                        but we had never seen it before",
                    );
                    let _ = empty.insert(true);
                }
                Entry::Occupied(mut filled) => {
                    #[cfg(any(test, debug_assertions))]
                    {
                        let old = filled.insert(true);
                        assert!(
                            !old,
                            "Tried to mark {sequent} proven, \
                        but we had already cached it as proven"
                        );
                    }
                    #[cfg(not(any(test, debug_assertions)))]
                    let _ = filled.insert(true);
                }
            }
            dbg_println!("    Proved {sequent}");
            Ok(())
        }
    }

    /// Check if we have a cached proof of this sequent.
    #[inline]
    pub(crate) fn proven(&self, sequent: &S) -> bool {
        let opt = self.cache.get(sequent);
        #[allow(unsafe_code)]
        // SAFETY:
        // Internal use only.
        // Called in one place that pulls from the queue of seen sequents anyway.
        *unsafe { opt.unwrap_unchecked() }
    }
}

impl<S: Sequent> Iterator for Thunk<S> {
    type Item = S;
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop().map(|Reverse(s)| s)
    }
}

impl<S: Sequent> Extend<S> for Thunk<S> {
    #[inline]
    #[allow(clippy::let_underscore_must_use)]
    fn extend<T: IntoIterator<Item = S>>(&mut self, iter: T) {
        for item in iter {
            let _ = self.push(item);
        }
    }
}
