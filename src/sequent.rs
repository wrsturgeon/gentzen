/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Anything that can represent a sequent,
//! i.e. a turnstile symbol with either nothing or
//! a comma-separated list of things on either side.

use crate::Infer;
use core::{
    fmt::{Debug, Display},
    hash::Hash,
};

/// Anything that can represent a sequent,
/// i.e. a turnstile symbol with either nothing or
/// a comma-separated list of things on either side.
pub trait Sequent: Clone + Debug + Display + Hash + Ord {
    /// Whatever is separated by commas on either side of a turnstile.
    type Item: Infer<Self>;
    /// Iterator that separates each unique element from everything else.
    // TODO: when `impl ...` is stabilized here, switch instead of building a vector
    // type Sampler = core::iter::Map<
    //     std::collections::btree_map::IntoKeys<Ast, NonZeroUsize>,
    //     fn(Ast) -> (Ast, Self),
    // >;
    /// Sequent with nothing on the left and this argument on the right.
    #[must_use]
    fn from_rhs(rhs_element: Self::Item) -> Self;
    /// For each unique item in the sequent (defined however you'd like),
    /// return a pair that separates that item from everything else.
    #[must_use]
    fn sample(&self) -> Vec<(Self::Item, Self)>;
}
