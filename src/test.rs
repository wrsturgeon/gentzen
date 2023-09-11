/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(
    clippy::arithmetic_side_effects,
    clippy::default_numeric_fallback,
    clippy::panic
)]

use crate::*;

#[test]
fn cant_prove_0() {
    assert_eq!(prove(Ast::Zero), Err(proof::Error::RanOutOfPaths));
}

#[test]
fn prove_1() {
    assert_eq!(prove(Ast::One), Ok(()));
}

#[test]
fn prove_top() {
    assert_eq!(prove(Ast::Top), Ok(()));
}

#[test]
fn prove_zero_par_top() {
    assert_eq!(prove(Ast::Zero.par(Ast::Top)), Ok(()));
}

#[test]
fn prove_0_implies_0() {
    assert_eq!(prove(Ast::Zero - Ast::Zero), Ok(()));
}

#[test]
fn prove_0_plus_1() {
    assert_eq!(prove(Ast::Zero + Ast::One), Ok(()));
}

#[test]
fn prove_1_plus_0() {
    assert_eq!(prove(Ast::One + Ast::Zero), Ok(()));
}

#[test]
fn prove_1_with_1() {
    assert_eq!(prove(Ast::One & Ast::One), Ok(()));
}

#[test]
fn prove_1_with_1_with_1() {
    assert_eq!(prove(Ast::One & Ast::One & Ast::One), Ok(()));
}

#[test]
fn prove_1_with_1_with_1_with_1() {
    assert_eq!(prove(Ast::One & Ast::One & Ast::One & Ast::One), Ok(()));
}

#[test]
fn prove_1_with_1_with_1_with_1_with_1() {
    assert_eq!(
        prove(Ast::One & Ast::One & Ast::One & Ast::One & Ast::One),
        Ok(())
    );
}

#[test]
fn cant_prove_0_with_1() {
    assert_eq!(
        prove(Ast::Zero & Ast::One),
        Err(proof::Error::RanOutOfPaths),
    );
}

#[test]
fn cant_prove_1_with_0() {
    assert_eq!(
        prove(Ast::One & Ast::Zero),
        Err(proof::Error::RanOutOfPaths),
    );
}

#[test]
fn a_with_b_implies_a() {
    assert_eq!(
        prove((Ast::Value(0) & Ast::Value(1)) - Ast::Value(0)),
        Ok(()),
    );
}

#[test]
fn a_with_b_implies_b() {
    assert_eq!(
        prove((Ast::Value(0) & Ast::Value(1)) - Ast::Value(1)),
        Ok(()),
    );
}

#[test]
fn bottom_implies_bottom() {
    assert_eq!(prove(Ast::Bottom - Ast::Bottom), Ok(()));
}

#[test]
fn prove_1_times_1() {
    assert_eq!(prove(Ast::One * Ast::One), Ok(()));
}

#[test]
fn cant_prove_1_times_0() {
    assert_eq!(
        prove(Ast::One * Ast::Zero),
        Err(proof::Error::RanOutOfPaths)
    );
}

#[test]
fn cant_prove_0_times_1() {
    assert_eq!(
        prove(Ast::Zero * Ast::One),
        Err(proof::Error::RanOutOfPaths)
    );
}

#[test]
fn cant_prove_0_times_0() {
    assert_eq!(
        prove(Ast::Zero * Ast::Zero),
        Err(proof::Error::RanOutOfPaths)
    );
}

#[test]
fn prove_1_times_1_times_1() {
    assert_eq!(prove(Ast::One * Ast::One * Ast::One), Ok(()));
}

#[test]
fn prove_1_times_1_times_1_times_1() {
    assert_eq!(prove(Ast::One * Ast::One * Ast::One * Ast::One), Ok(()));
}

#[test]
fn prove_1_times_1_times_1_times_1_times_1() {
    assert_eq!(
        prove(Ast::One * Ast::One * Ast::One * Ast::One * Ast::One),
        Ok(())
    );
}

#[test]
fn prove_1_implies_1_implies_1_implies_1_implies_1_times_1() {
    assert_eq!(
        prove(Ast::One - (Ast::One - (Ast::One - (Ast::One - (Ast::One * Ast::One))))),
        Ok(())
    );
}

#[test]
fn prove_excluded_middle_par() {
    assert_eq!(prove(Ast::Value(0).par(-Ast::Value(0))), Ok(()));
}

#[test]
fn cant_prove_excluded_middle_plus() {
    assert_eq!(
        prove(Ast::Value(0) + -Ast::Value(0)),
        Err(proof::Error::RanOutOfPaths)
    );
}

#[test]
fn cant_prove_excluded_middle_with() {
    assert_eq!(
        prove(Ast::Value(0) & -Ast::Value(0)),
        Err(proof::Error::RanOutOfPaths)
    );
}

#[cfg(feature = "quickcheck")]
quickcheck::quickcheck! {
    // fn trace_eq_implies_equal_hashes(a: Trace, b: Trace) -> bool {
    //     eq_implies_hash(&a, &b)
    // }

    // fn split_eq_implies_equal_hashes(a: Split, b: Split) -> bool {
    //     eq_implies_hash(&a, &b)
    // }
}

mod reduced {
    // use super::*;

    // #[test]
    // fn sorted_after_sort_1() {
    //     assert_eq!((Ast::One * Ast::One).sort().sorted(), Ok(()));
    // }

    // #[test]
    // fn sorted_after_sort_2() {
    //     assert_eq!((Ast::Bottom * Ast::One).sort().sorted(), Ok(()));
    // }

    // #[test]
    // fn sorted_after_sort_3() {
    //     assert_eq!((Ast::One * Ast::Bottom).sort().sorted(), Ok(()));
    // }

    // #[test]
    // fn sorted_after_sort_4() {
    //     assert_eq!(((Ast::One * Ast::One) * Ast::One).sort().sorted(), Ok(()));
    // }

    // #[test]
    // fn sorted_after_sort_5() {
    //     let pre = Ast::One * (Ast::One.par(Ast::One));
    //     print!("[{pre}] --> ");
    //     let post = pre.sort();
    //     println!("[{post}]");
    //     assert_eq!(post.sorted(), Ok(()));
    // }

    // #[test]
    // #[allow(unsafe_code)]
    // fn split_swap_still_equal_hashes_1() {
    //     use crate::{
    //         {Split, Trace},
    //         Ast, Multiset, trace,
    //     };
    //     use core::{
    //         hash::{Hash, Hasher},
    //         num::NonZeroUsize,
    //     };
    //     use std::collections::{hash_map::DefaultHasher, BTreeMap};
    //     let mut btm = BTreeMap::new();
    //     // SAFETY: duh
    //     let _ = btm.insert(Ast::One, unsafe {
    //         NonZeroUsize::new_unchecked(29_715_618_991_585_221)
    //     });
    //     // SAFETY: duh
    //     let _ = btm.insert(Ast::Bottom, unsafe {
    //         NonZeroUsize::new_unchecked(18_417_028_454_717_966_395)
    //     });
    //     let split = Split {
    //         lhs: Trace {
    //             current: trace {
    //                 rhs: Multiset(BTreeMap::new()),
    //             },
    //             history: None,
    //         },
    //         rhs: Trace {
    //             current: trace { rhs: Multiset(btm) },
    //             history: None,
    //         },
    //     };
    //     let &Split { ref lhs, ref rhs } = &split;
    //     let swap = Split {
    //         lhs: rhs.clone(),
    //         rhs: lhs.clone(),
    //     };
    //     assert_eq!(split, swap);
    //     let mut h = DefaultHasher::new();
    //     split.hash(&mut h);
    //     let hash_a = h.finish();
    //     h = DefaultHasher::new();
    //     swap.hash(&mut h);
    //     let hash_b = h.finish();
    //     assert_eq!(hash_a, hash_b);
    // }
}

// #[inline]
// #[cfg(feature = "quickcheck")]
// fn eq_implies_hash<T: Eq + core::hash::Hash>(a: &T, b: &T) -> bool {
//     use {core::hash::Hasher, std::collections::hash_map::DefaultHasher};
//     if a != b {
//         return true;
//     }
//     let mut h = DefaultHasher::new();
//     a.hash(&mut h);
//     let hash_a = h.finish();
//     h = DefaultHasher::new();
//     b.hash(&mut h);
//     let hash_b = h.finish();
//     hash_a == hash_b
// }
