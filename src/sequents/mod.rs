/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Various common sequent structures to avoid reinventing the wheel.

mod intuitionist_with_exchange;
mod rhs_only_with_exchange;

pub use {
    intuitionist_with_exchange::IntuitionistWithExchange,
    rhs_only_with_exchange::RhsOnlyWithExchange,
};
