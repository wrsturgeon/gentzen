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
    turnstile::{Split, Trace},
    Ast, Multiset, Turnstile,
};
use core::cmp::Reverse;
use std::{
    collections::{hash_map::Entry, BinaryHeap, HashMap, HashSet},
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

impl From<bool> for State {
    #[inline(always)]
    fn from(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }
}

/// Unsuccessful proof.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    /// Ran out of actionable turnstiles to manipulate.
    RanOutOfPaths,
}

/// Inference: either the proof is done, we have another turnstile, or we have to prove two.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Inference {
    /// Proof is done: nothing above the inference line.
    Qed(Rc<Trace>),
    /// One turnstile above the inference line.
    Linear(Trace),
    /// Two turnstiles above the inference line.
    Binary(Split),
    /// Contradiction: nothing above the inference line.
    Contradiction(Rc<Trace>),
}

impl Inference {
    /// Update a proof in response to an inference.
    #[inline]
    #[allow(unsafe_code)]
    #[cfg_attr(debug_assertions, allow(clippy::print_stdout))]
    fn handle(
        self,
        seen: &mut HashMap<Turnstile, State>,
        paths: &mut BinaryHeap<Reverse<Trace>>,
        paused: &mut HashSet<Split>,
        original: &Turnstile,
    ) -> bool {
        match self {
            Inference::Qed(history) => {
                if cache_proof(seen, paused, &history, true, original).is_none() {
                    return true;
                }
            }
            Inference::Contradiction(history) => {
                if cache_proof(seen, paused, &history, false, original).is_none() {
                    return true;
                }
            }
            Inference::Linear(trace) => add_if_new(seen, paths, trace),
            Inference::Binary(split) => {
                for turnstile in &split {
                    add_if_new(
                        seen,
                        paths,
                        Trace {
                            current: turnstile.clone(),
                            history: None,
                        },
                    );
                }
                match split.proven(seen) {
                    State::True => {
                        if cache_proof(seen, paused, &split.history, true, original).is_none() {
                            return true;
                        }
                    }
                    State::False => {
                        if cache_proof(seen, paused, &split.history, false, original).is_none() {
                            return true;
                        }
                    }
                    State::Unknown => {
                        #[cfg(debug_assertions)]
                        if paused.insert(split.clone()) {
                            println!("    Pausing {split}");
                        }
                        #[cfg(not(debug_assertions))]
                        let _ = paused.insert(split.clone());
                    }
                }
            }
        }
        false
    }
}

/// Step through each turnstile in this proof and mark it with the provided state.
#[inline]
#[allow(unsafe_code)]
#[cfg_attr(debug_assertions, allow(clippy::print_stdout))]
fn cache_proof(
    seen: &mut HashMap<Turnstile, State>,
    paused: &mut HashSet<Split>,
    mut trace: &Rc<Trace>,
    truth: bool,
    original: &Turnstile,
) -> Option<()> {
    let state = truth.into();
    loop {
        #[cfg(debug_assertions)]
        println!(
            "    {}roved {}",
            if truth { "P" } else { "Disp" },
            trace.current,
        );
        if &trace.current == original {
            return None;
        }
        // SAFETY: Had to be added to `seen` before extended to another turnstile.
        let current = unsafe { seen.get_mut(&trace.current).unwrap_unchecked() };
        // debug_assert_eq!(
        //     current,
        //     &mut State::Unknown,
        //     "Claimed to have proven a statement that was already {current:?}"
        // );
        *current = state;
        if let Some(ref next) = trace.history {
            trace = next;
        } else {
            break;
        }
    }
    let mut remove = vec![];
    for split in &*paused {
        match split.proven(seen) {
            State::True => {
                remove.push((split.clone(), true));
            }
            State::False => {
                remove.push((split.clone(), false));
            }
            State::Unknown => { /* fall through */ }
        }
    }
    for &(ref split, _) in &remove {
        assert!(
            paused.remove(split),
            "Trying to remove a value from `paused` that wasn't in it"
        );
    }
    for (split, proven) in remove {
        cache_proof(seen, paused, &split.history, proven, original)?;
    }
    Some(())
}

/// Add this turnstile to the queue iff we haven't seen it before.
#[inline]
#[cfg_attr(debug_assertions, allow(clippy::print_stdout))]
fn add_if_new(
    seen: &mut HashMap<Turnstile, State>,
    paths: &mut BinaryHeap<Reverse<Trace>>,
    trace: Trace,
) {
    match seen.entry(trace.current.clone()) {
        Entry::Vacant(unseen) => {
            let _ = unseen.insert(State::Unknown);
            #[cfg(debug_assertions)]
            println!("    Adding {trace}");
            paths.push(Reverse(trace));
        }
        Entry::Occupied(_) => { /* fall through */ }
    }
}

#[allow(clippy::multiple_inherent_impl)]
impl Ast {
    /// Attempt to prove this expression with sequent-calculus proof search.
    /// # Errors
    /// If we can't.
    #[inline]
    #[cfg_attr(debug_assertions, allow(clippy::print_stdout))]
    pub fn prove(self) -> Result<(), Error> {
        let original = Turnstile::new(self);
        let mut seen = HashMap::new();
        let mut paths = BinaryHeap::new();
        let mut paused: HashSet<Split> = HashSet::new();
        let _ = seen.insert(original.clone(), State::Unknown);
        paths.push(Reverse(Trace {
            current: original.clone(),
            history: None,
        }));
        while let Some(Reverse(path)) = paths.pop() {
            #[cfg(debug_assertions)]
            println!("Testing {path}");
            for expansion in Rc::new(path).infer() {
                if expansion.handle(&mut seen, &mut paths, &mut paused, &original) {
                    return Ok(());
                }
            }
        }
        Err(Error::RanOutOfPaths)
    }
}

#[allow(clippy::multiple_inherent_impl)]
impl Trace {
    /// All possible "next moves" in a sequent-calculus proof search.
    #[inline(always)]
    pub(crate) fn infer(self: &Rc<Self>) -> Vec<Inference> {
        if self.current.rhs.contains(&Ast::Top) {
            return vec![Inference::Qed(Rc::clone(self))];
        }
        match self.current.rhs.only() {
            Some(&Ast::One) => return vec![Inference::Qed(Rc::clone(self))],
            Some(&Ast::Bottom) => return vec![Inference::Contradiction(Rc::clone(self))],
            Some(_) | None => {}
        }
        match self.current.rhs.pair() {
            Some((&Ast::Dual(ref neg), pos) | (pos, &Ast::Dual(ref neg))) => {
                if neg.as_ref() == pos {
                    return vec![Inference::Qed(Rc::clone(self))];
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
                ast.infer(ablation, self)
            })
            .collect()
    }

    /// Continue with a single turnstile above the inference line.
    #[must_use]
    #[inline(always)]
    pub fn one(self: &Rc<Self>, child: Multiset<Ast>) -> Inference {
        #[cfg(test)]
        self.assert_acyclic();
        Inference::Linear(Self {
            current: Turnstile { rhs: child },
            history: Some(Rc::clone(self)),
        })
    }

    /// Continue with two children.
    #[must_use]
    #[inline(always)]
    pub fn two(self: &Rc<Self>, lhs: Multiset<Ast>, rhs: Multiset<Ast>) -> Inference {
        #[cfg(test)]
        self.assert_acyclic();
        Inference::Binary(Split {
            turnstiles: [Turnstile { rhs: lhs }, Turnstile { rhs }]
                .into_iter()
                .collect(),
            history: Rc::clone(self),
        })
    }

    #[cfg(test)]
    fn assert_acyclic(self: &Rc<Self>) {
        self.assert_acyclic_helper(&mut vec![]);
    }

    #[cfg(test)]
    #[allow(clippy::arithmetic_side_effects)]
    fn assert_acyclic_helper<'s>(self: &'s Rc<Self>, visited: &mut Vec<&'s Rc<Self>>) {
        assert!(
            !visited.contains(&self),
            "Cyclic proof: {} {self}",
            visited.iter_mut().fold(String::new(), |acc, trace| acc
                + &trace.to_string()
                + " -> ")
        );
        visited.push(self);
        if let Some(ref trace) = self.history {
            trace.assert_acyclic_helper(visited);
        }
    }
}

#[allow(clippy::multiple_inherent_impl)]
impl Ast {
    /// All possible "next moves" in a sequent-calculus proof search.
    #[inline]
    pub(crate) fn infer(
        &self,
        context: Multiset<Ast>,
        parent: &Rc<Trace>,
    ) -> impl IntoIterator<Item = Inference> {
        match self {
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
            &Self::Dual(ref dual) => vec![parent.one(context.with([match dual.as_ref() {
                &Self::One => Self::Bottom,
                &Self::Bottom => Self::One,
                &Self::Top => Self::Zero,
                &Self::Zero => Self::Top,
                &Self::Value(_) => return vec![],
                &Self::Bang(ref arg) => Self::Quest(Box::new(Self::Dual(arg.clone()))),
                &Self::Quest(ref arg) => Self::Bang(Box::new(Self::Dual(arg.clone()))),
                &Self::Dual(ref arg) => arg.as_ref().clone(),
                &Self::Times(ref lhs, ref rhs) => Self::Par(
                    Box::new(Self::Dual(lhs.clone())),
                    Box::new(Self::Dual(rhs.clone())),
                ),
                &Self::Par(ref lhs, ref rhs) => Self::Times(
                    Box::new(Self::Dual(lhs.clone())),
                    Box::new(Self::Dual(rhs.clone())),
                ),
                &Self::With(ref lhs, ref rhs) => Self::Plus(
                    Box::new(Self::Dual(lhs.clone())),
                    Box::new(Self::Dual(rhs.clone())),
                ),
                &Self::Plus(ref lhs, ref rhs) => Self::With(
                    Box::new(Self::Dual(lhs.clone())),
                    Box::new(Self::Dual(rhs.clone())),
                ),
            }]))],
            &Self::Times(ref blhs, ref brhs) => {
                let lhs = blhs.as_ref();
                let rhs = brhs.as_ref();
                let power_of_2 = 1_usize
                    .checked_shl(context.len().try_into().expect("Ridiculously huge value"))
                    .expect("More elements in a sequent than bits in a `usize`");
                (0..power_of_2)
                    .flat_map(|bits| {
                        let (mut lctx, mut rctx) = (Multiset::new(), Multiset::new());
                        for (i, ast) in context.iter().enumerate() {
                            let _ = if bits & (1 << i) == 0 {
                                &mut lctx
                            } else {
                                &mut rctx
                            }
                            .insert(ast.clone());
                        }
                        [
                            parent.two(lctx.with([lhs.clone()]), rctx.with([rhs.clone()])),
                            parent.two(rctx.with([lhs.clone()]), lctx.with([rhs.clone()])),
                        ]
                    })
                    .collect()
            }
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
        }
    }
}

#[allow(clippy::multiple_inherent_impl)]
impl Split {
    /// Check if we have proofs *cached* that establish the truth or falsity of a set of sequents.
    #[inline]
    #[cfg_attr(debug_assertions, allow(clippy::print_stdout))]
    fn proven(&self, seen: &HashMap<Turnstile, State>) -> State {
        let mut all_proven = true;
        for turnstile in self {
            #[allow(unsafe_code)]
            // SAFETY: Always added to `seen` before being traced.
            match unsafe { seen.get(turnstile).unwrap_unchecked() } {
                &State::False => {
                    #[cfg(debug_assertions)]
                    println!("    Disproved {self}");
                    return State::False;
                }
                &State::Unknown => all_proven = false,
                &State::True => {}
            }
        }
        if all_proven {
            #[cfg(debug_assertions)]
            println!("    Proved {}", self.without_history());
            State::True
        } else {
            State::Unknown
        }
    }
}
