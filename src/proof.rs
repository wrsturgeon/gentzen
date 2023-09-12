/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Proofs by sequent-calculus proof search.

// One-sided sequent calculus presentation:
//
// ----- Initial sequent
// A, ~A
//
// |- G, A   |- ~A, D
// ------------------ Cut
// |- G, D
//
// |- G, A, B, D
// ------------- Exchange (but not contraction, so we can use a multiset)
// |- G, B, A, D
//
// |- G, A   |- D, B
// ----------------- Multiplicative conjunction
// |- G, D, A * B
//
// |- G, A, B
// ------------- Multiplicative disjunction
// |- G, A par B
//
// ---- Unit for multiplicative conjunction
// |- 1
//
// |- G
// ------------ Unit for multiplicative disjunction
// |- G, Bottom
//
// |- G, A   |- G, B
// ----------------- Additive conjunction
// |- G, A & B
//
// |- G, A
// ----------- Additive disjunction (left operand)
// |- G, A + B
//
// |- G, B
// ----------- Additive disjunction (right operand)
// |- G, A + B
//
// --------- Unit for additive conjunction
// |- G, Top
//
// (no rule for 0, the unit for additive disjunction)
//
// |- G
// -------- Weakening for why-not exponential
// |- G, ?A
//
// |- G, ?A, ?A
// ------------ Contraction for why-not exponential
// |- G, ?A
//
// |- ?G, A
// --------- Exponential rule #1
// |- ?G, !A
//
// |- G, A
// -------- Exponential rule #2
// |- G, ?A
//
// That's all, folks!
//
// |- G, ~A par B
// -------------- Lollipop definition
// |- G, A -* B
//
//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
//
// Two-sided sequent calculus presentation:
//
// ------ init
// B |- B
//
// ---- 1R
// |- 1
//
// --------- bottom L
// bottom |-
//
// --------- 0L
// D, 0 |- G
//
// ----------- top R
// D |- top, G
//
// D1 |- B, G1   D2, B |- G2
// ------------------------- cut
// D1, D2 |- G1, G2
//
// D |- B, G
// ---------- ~L
// D, ~B |- G
//
// D, B |- G
// ---------- ~R
// D |- ~B, G
//
// D |- G
// --------- 1L
// D, 1 |- G
//
// D |- G
// -------------- bottom R
// D |- bottom, G
//
// D, B1, B2, |- G
// --------------- *L
// D, B1 * B2 |- G
//
// D1 |- B, G1   D2 |- C, G2
// ------------------------- *R
// D1, D2 |- B * C, G1, G2
//
// D1, B |- G1   D2, C |- G2
// ------------------------- par L
// D1, D2, B par C |- G1, G2
//
// D |- B, C, G
// --------------- par R
// D |- B par C, G
//
// D, B1 |- G
// --------------- &L1
// D, B1 & B2 |- G
//
// D, B2 |- G
// --------------- &L2
// D, B1 & B2 |- G
//
// D |- B, G   D |- C, G
// --------------------- &R
// D |- B & C, G
//
// D, B |- G   D, C |- G
// --------------------- +L
// D, B + C |- G
//
// D |- B1, G
// --------------- +R1
// D |- B1 + B2, G
//
// D |- B2, G
// --------------- +R1
// D |- B1 + B2, G
//
// D, B[t/x] |- G
// ------------------- forall L
// D, forall x. B |- G
//
// D |- B[y/x], G
// ------------------- forall R
// D |- forall x. B, G
//
// D, B[y/x] |- G
// ------------------- exists L
// D, exists x. B |- G
//
// D |- B[t/x], G
// -------------------- exists R
// D |- exists x. (B G)
//
// D |- G
// ---------- !W
// D, !B |- G
//
// D, !B, !B |- G
// -------------- !C
// D, !B |- G
//
// D, B |- G
// ---------- !D
// D, !B |- G
//
// D |- G
// ---------- ?W
// D |- ?B, G
//
// D |- ?B, ?B, G
// -------------- ?C
// D |- ?B, G
//
// D |- B, G
// ---------- ?D
// D |- ?B, G
//
// !D, B |- ?G
// ------------ ?L
// !D, ?B |- ?G
//
// !D |- B, ?G
// ------------ !R
// !D |- !B, ?G

use crate::{
    inference::Inference,
    thunk::{Qed, Thunk},
    Infer, Rule, Sequent, Tree,
};
use core::hash::Hash;
use std::{collections::HashSet, rc::Rc};

/// Unsuccessful proof.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    /// Ran out of actionable sequents to manipulate.
    RanOutOfPaths,
}

/// Attempt to prove this expression with sequent-calculus proof search.
/// # Errors
/// If we can't.
#[inline]
pub fn prove<I: Infer<S>, S: Sequent<Item = I>>(expr: I) -> Result<Tree<S>, Error> {
    let mut queue: Thunk<S> = Thunk::new(expr.clone());
    let mut paused = HashSet::new();
    while let Some(sequent) = queue.next() {
        dbg_println!("Trying {sequent}");
        let rc = Rc::new(sequent);
        for inference in rc
            .sample()
            .into_iter()
            .flat_map(move |(item, context)| item.above(context))
            .map(|rule| Inference {
                rule,
                below: Rc::clone(&rc),
            })
        {
            // dbg_println!("    Pausing {inference}");
            let sequents = inference.rule.above.clone();
            let _ = paused.insert(inference);
            queue.extend(sequents);
        }
        let mut done = HashSet::new();
        'inferences: loop {
            for inference in &paused {
                if !done.contains(inference) && inference.proven(&queue) {
                    dbg_println!("    Proved {inference}");
                    match queue.cache(inference.below.as_ref().clone(), inference.rule.clone()) {
                        Ok(()) => {
                            let _ = done.insert(inference.clone());
                            continue 'inferences;
                        }
                        Err(Qed {
                            proof: Rule { name, above },
                        }) => return Ok(Tree::connect(S::from_rhs(expr), name, above, &mut queue)),
                    };
                }
            }
            break 'inferences;
        }
        for inference in &done {
            let _ = paused.remove(inference);
        }
    }
    Err(Error::RanOutOfPaths)
}
