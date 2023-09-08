/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::arithmetic_side_effects, clippy::panic, clippy::print_stdout)]

use crate::*;
use quickcheck::quickcheck;

#[test]
fn cant_prove_0() {
    assert_eq!(Ast::Zero.prove(), Err(proof::Error::RanOutOfPaths));
}

// #[test]
// fn cant_prove_0_times_1() {
//     assert_eq!(
//         (Ast::Zero * Ast::One).prove(),
//         Err(proof::Error::RanOutOfPaths),
//     );
// }

#[test]
fn prove_1() {
    assert_eq!(Ast::One.prove(), Ok(()));
}

#[test]
fn prove_0_implies_0() {
    assert_eq!((Ast::Zero - Ast::Zero).prove(), Ok(()));
}

// #[test]
// fn bang_a_implies_a() {
//     assert_eq!((bang(Ast::Value(0)) - Ast::Value(0)).prove(), Ok(()));
// }

// #[test]
// fn a_with_b_implies_a() {
//     assert_eq!(
//         ((Ast::Value(0) & Ast::Value(1)) - Ast::Value(0)).prove(),
//         Ok(()),
//     );
// }

// #[test]
// fn a_with_b_implies_b() {
//     assert_eq!(
//         ((Ast::Value(0) & Ast::Value(1)) - Ast::Value(1)).prove(),
//         Ok(()),
//     );
// }

quickcheck! {
    // fn trace_eq_implies_equal_hashes(a: turnstile::Trace, b: turnstile::Trace) -> quickcheck::TestResult {
    //     use {core::hash::{Hash, Hasher}, std::collections::hash_map::DefaultHasher as DefaultHasher};
    //     if !a.eq(&b) {
    //         return quickcheck::TestResult::discard();
    //     }
    //     let mut h = DefaultHasher::new();
    //     a.hash(&mut h);
    //     let hash_a = h.finish();
    //     h = DefaultHasher::new();
    //     b.hash(&mut h);
    //     let hash_b = h.finish();
    //     quickcheck::TestResult::from_bool(hash_a.eq(&hash_b))
    // }

    fn infix_one_to_one(infix: ast::Infix) -> bool { infix.into_ast(Box::new(Ast::Zero), Box::new(Ast::Zero)).infix_op() == Some(infix) }

    // #[allow(clippy::double_neg)]
    // fn involutive_dual(ast: Ast) -> bool { (--ast.clone()) == ast }

    // fn sorted_after_sort(ast: Ast) -> bool { ast.sort().sorted() == Ok(()) }
    // fn sorted_invariant_over_sort(ast: Ast) -> bool {
    //     let sorted = ast.sort();
    //     sorted.clone().sort() == sorted
    // }

    // fn sorted_after_bang(ast: Ast) -> bool { bang(ast.sort()).sorted() == Ok(()) }
    // fn sorted_after_quest(ast: Ast) -> bool { quest(ast.sort()).sorted() == Ok(()) }
    // fn sorted_after_dual(ast: Ast) -> bool { (-(ast.sort())).sorted() == Ok(()) }
    // fn sorted_after_times(lhs: Ast, rhs: Ast) -> bool { (lhs.sort() * rhs.sort()).sorted() == Ok(()) }
    // fn sorted_after_par(lhs: Ast, rhs: Ast) -> bool { lhs.sort().par(rhs.sort()).sorted() == Ok(()) }
    // fn sorted_after_with(lhs: Ast, rhs: Ast) -> bool { (lhs.sort() & rhs.sort()).sorted() == Ok(()) }
    // fn sorted_after_plus(lhs: Ast, rhs: Ast) -> bool { (lhs.sort() + rhs.sort()).sorted() == Ok(()) }
    // fn sorted_after_lollipop(lhs: Ast, rhs: Ast) -> bool { (lhs.sort() - rhs.sort()).sorted() == Ok(()) }
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
}
