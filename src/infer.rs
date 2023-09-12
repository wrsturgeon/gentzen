/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A set of sequent-calculus inference rules using the specified sequent structure.

use crate::{Rule, Sequent};

/// A set of sequent-calculus inference rules using the specified sequent structure.
pub trait Infer<S: Sequent<Item = Self>>: Clone {
    /// All possible "next moves" in a sequent-calculus proof search.
    /// Note that each item in the resultant `HashSet` is a _separate_ inference line:
    /// if you want to place multiple sequents above a single inference line,
    /// use `below.require_all([first, second, ...])`.
    fn above(&self, context: S) -> Vec<Rule<S>>;
}
