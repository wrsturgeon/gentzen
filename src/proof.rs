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

use crate::{turnstile::Trace, Ast, Multiset, Turnstiles};
use core::cmp::Reverse;
use std::{
    collections::{BTreeSet, BinaryHeap, HashMap},
    rc::Rc,
};

/// Either known to be true/false or still working on it.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum State {
    /// Known to be false.
    False,
    /// Known to be true.
    True,
    /// Still working on it.
    Unknown,
}

/// Unsuccessful proof.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    /// Ran out of actionable turnstiles to manipulate.
    RanOutOfPaths,
}

/// Attempt to prove a statement by sequent-calculus proof search.
#[allow(clippy::print_stdout)] // FIXME: remove `println!`s
pub(crate) fn prove(ast: Ast) -> Result<(), Error> {
    let mut seen = HashMap::new();
    let mut paths = BinaryHeap::new();
    {
        let init = Trace::from_thin_air(ast);
        let _ = seen.insert(&init, State::Unknown);
        paths.push(Reverse(Turnstiles::new(init)));
    }
    while let Some(Reverse(turnstiles)) = paths.pop() {
        println!("Testing {turnstiles}");
        let trace = match turnstiles.only() {
            Ok(sole_element) => sole_element,
            Err(proven) => return proven.then_some(()).ok_or(Error::RanOutOfPaths),
        };
        for next_turnstiles in Rc::new(trace).expand() {
            // TODO: pause iff ANY individual turnstile has been seen
            // for branch in &next_step {
            //     println!("    Adding {branch}");
            //     // match seen.entry(&branch) {
            //     //     Entry::Vacant(unseen) => {
            //     //         unseen.insert(State::Unknown);
            //     //         paths.push(Reverse(next_step))
            //     //     }
            //     //     Entry::Occupied(_already_working_on_it) => {}
            //     // }
            //     todo!()
            // }
            paths.push(Reverse(next_turnstiles));
        }
    }
    Err(Error::RanOutOfPaths)
}

#[allow(clippy::multiple_inherent_impl)]
impl Trace {
    /// All possible "next moves" in a sequent-calculus proof search.
    #[inline(always)]
    pub(crate) fn expand(self: &Rc<Self>) -> impl IntoIterator<Item = Turnstiles> {
        if self.current.rhs.contains(&Ast::Top) {
            return vec![Turnstiles::qed()];
        }
        match self.current.rhs.only() {
            Some(&Ast::One) => return vec![Turnstiles::qed()],
            Some(_) | None => {}
        }
        match self.current.rhs.pair() {
            Some((&Ast::Dual(ref neg), pos) | (pos, &Ast::Dual(ref neg))) => {
                if neg.as_ref() == pos {
                    return vec![Turnstiles::qed()];
                }
            }
            Some(_) | None => {}
        }
        self.current
            .rhs
            .iter_unique()
            .flat_map(|(ast, _)| {
                let mut ablation = self.current.rhs.clone();
                let _ = ablation.take(ast);
                ast.expand(ablation, self)
            })
            .collect()
    }
}

#[allow(clippy::multiple_inherent_impl)]
impl Ast {
    /// All possible "next moves" in a sequent-calculus proof search.
    #[inline]
    pub(crate) fn expand(
        &self,
        context: Multiset<Ast>,
        parent: &Rc<Trace>,
    ) -> impl IntoIterator<Item = Turnstiles> {
        let v: Vec<BTreeSet<Trace>> = match self {
            &(Self::One | Self::Top | Self::Zero | Self::Value(_)) => vec![],
            &Self::Bottom => vec![parent.one(context)],
            &Self::Bang(ref arg) => {
                if matches!(context.only(), Some(&Self::Quest(_))) {
                    vec![parent.one(context.with([arg.as_ref().clone()]))]
                } else {
                    vec![]
                }
            }
            &Self::Quest(ref arg) => vec![
                parent.one(context.clone()),
                parent.one(context.with([arg.as_ref().clone()])),
                parent.one(context.with([Self::Quest(arg.clone()), Self::Quest(arg.clone())])),
            ],
            &Self::Dual(ref _arg) => todo!(),
            &Self::Times(ref _lhs, ref _rhs) => todo!(),
            &Self::Par(ref lhs, ref rhs) => {
                vec![parent.one(context.with([lhs.as_ref().clone(), rhs.as_ref().clone()]))]
            }
            &Self::With(ref lhs, ref rhs) => {
                vec![parent.two(
                    context.with([lhs.as_ref().clone()]),
                    context.with([rhs.as_ref().clone()]),
                )]
            }
            &Self::Plus(ref lhs, ref rhs) => vec![
                parent.one(context.with([lhs.as_ref().clone()])),
                parent.one(context.with([rhs.as_ref().clone()])),
            ],
        };
        v.into_iter().map(Turnstiles)
    }
}
