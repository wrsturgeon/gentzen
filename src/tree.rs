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
                            below: sequent.clone(),
                        },
                        |Rule { name, above }| Tree::connect(sequent, name, above, thunk),
                    )
                })
                .collect(),
            rule,
            below,
        }
    }

    /// Each line of printed output.
    pub(crate) fn print_bottom_up(&self) -> (Vec<String>, usize) {
        let mut columns: Vec<_> = self
            .above
            .iter()
            .map(|tree| {
                let (v, line_width) = tree.print_bottom_up();
                let entire_width = {
                    #[allow(unsafe_code)]
                    // SAFETY: Base case 2 lines, each iteration lengthens, so always nonzero
                    unsafe {
                        v.iter().map(|s| s.chars().count()).max().unwrap_unchecked()
                    }
                };
                (v, line_width, entire_width)
            })
            .collect();
        columns.sort_by_key(|&(_, _, entire_width)| entire_width);
        let (line_size, maybe_stack) = columns.pop().map_or((0, None), |rightmost| {
            let mut overall_width = 0;
            let mut v = vec![];
            for (stack, _, entire_width) in columns {
                extend_upward(&mut v, stack, overall_width);
                overall_width = overall_width.saturating_add(entire_width).saturating_add(3);
            }
            let (stack, line_width, _) = rightmost;
            extend_upward(&mut v, stack, overall_width);
            (overall_width.saturating_add(line_width), Some(v))
        });
        let below = self.below.to_string();
        let max_width = line_size.max(below.chars().count());
        let mut line = String::new();
        for _ in 0..max_width {
            line.push('-');
        }
        line.push(' ');
        line.push_str(self.rule);
        let mut everything = vec![below, line];
        if let Some(stack) = maybe_stack {
            everything.extend(stack);
        }
        (everything, max_width)
    }
}

/// Add a column to a print of a proof, even if the previous print wasn't tall enough.
#[inline]
#[allow(clippy::option_if_let_else)] // Mutable borrow issues with `Option::map_or_else`
fn extend_upward(v: &mut Vec<String>, stack: Vec<String>, overall_width: usize) {
    for (i, line) in stack.into_iter().enumerate() {
        let acc = if let Some(s) = v.get_mut(i) {
            s
        } else {
            v.push(" ".repeat(overall_width));
            #[allow(unsafe_code)]
            // SAFETY:
            // Iterating one at a time from zero,
            // so we can never be less than one element behind,
            // and we just added one above.
            unsafe {
                v.get_unchecked_mut(i)
            }
        };
        for _ in acc.chars().count()..overall_width {
            acc.push(' ');
        }
        acc.push_str(&line);
    }
}

impl<S: Sequent> core::fmt::Display for Tree<S> {
    #[inline]
    #[allow(clippy::arithmetic_side_effects)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f)?;
        for line in self.print_bottom_up().0.into_iter().rev() {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}
