/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Proof as a tree rooted at the bottom (the original expression).

use crate::{thunk::Thunk, Rule, Sequent};
use std::collections::BTreeSet;

/// Proof as a tree rooted at the bottom (the original expression).
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Tree<S: Sequent> {
    /// Proof of each sequent above the inference line.
    pub above: BTreeSet<Self>,
    /// Name of the rule that allowed this inference.
    pub rule: &'static str,
    /// Sequent below the inference line (proven by those above).
    pub below: S,
}

impl<S: Sequent> Tree<S> {
    /// Chain cached proof steps together into a single proof.
    #[inline]
    pub(crate) fn connect<Above: IntoIterator<Item = S>>(
        below: S,
        rule: &'static str,
        next: Above,
        thunk: &mut Thunk<S>,
    ) -> Self {
        Tree {
            above: next
                .into_iter()
                .map(|sequent| {
                    thunk.yank(&sequent).map_or(
                        Tree {
                            above: BTreeSet::new(),
                            rule: "(already proven)",
                            below: below.clone(),
                        },
                        |Rule { name, above }| Tree::connect(sequent, name, above, thunk),
                    )
                })
                .collect(),
            rule,
            below,
        }
    }
}
