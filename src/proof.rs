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

impl State {
    /// Whether this is `State::True`.
    #[inline(always)]
    pub(crate) const fn proven(self) -> bool {
        matches!(self, State::True)
    }
}

/// Unsuccessful proof.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    /// Ran out of actionable turnstiles to manipulate.
    RanOutOfPaths,
}

/// Expansion: either the proof is done, we have another turnstile, or we have to prove two.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Expansion {
    /// Proof is done: nothing above the inference line.
    Qed(Rc<Trace>),
    /// One turnstile above the inference line.
    Linear(Trace),
    /// Two turnstiles above the inference line.
    Binary(Split),
    /// Contradiction: nothing above the inference line.
    Contradiction(Rc<Trace>),
}

/// Step through each turnstile in this proof and mark it with the provided state.
#[inline]
#[allow(unsafe_code)]
#[allow(clippy::print_stdout)] // FIXME: remove `println!`s
fn cache_proof(
    seen: &mut HashMap<Turnstile, State>,
    paused: &HashSet<Split>,
    mut trace: &Rc<Trace>,
    truth: bool,
    original: &Turnstile,
) -> Option<impl IntoIterator<Item = Split>> {
    let state = truth.into();
    loop {
        println!(
            "    {}roved {}",
            if truth { "P" } else { "Disp" },
            trace.current
        );
        if &trace.current == original {
            return None;
        }
        // SAFETY: Had to be added to `seen` before extended to another turnstile.
        let current = unsafe { seen.get_mut(&trace.current).unwrap_unchecked() };
        debug_assert_eq!(
            current,
            &mut State::Unknown,
            "Claimed to have proven a statement that was already {current:?}"
        );
        *current = state;
        if let Some(ref next) = trace.history {
            trace = next;
        } else {
            break;
        }
    }
    let mut proven = HashSet::new();
    for split in paused {
        // SAFETY: Always added to `seen` before `paused`.
        match unsafe {
            (
                seen.get(&split.lhs.current).unwrap_unchecked(),
                seen.get(&split.rhs.current).unwrap_unchecked(),
            )
        } {
            (&State::True, &State::True) => {
                let (lhs, rhs) = split.sort();
                println!("    Proved [ {lhs}   {rhs} ]");
                let _ = proven.insert(split.clone());
                proven.extend(cache_proof(seen, paused, &split.history, true, original)?);
            }
            (&State::False, _) | (_, &State::False) => {
                let (lhs, rhs) = split.sort();
                println!("    Disproved [ {lhs}   {rhs} ]");
                let _ = proven.insert(split.clone());
                proven.extend(cache_proof(seen, paused, &split.history, false, original)?);
            }
            _ => { /* fall through */ }
        }
    }
    Some(proven)
}

/// Add this turnstile to the queue iff we haven't seen it before.
#[inline]
#[allow(clippy::print_stdout)] // FIXME: remove `println!`s
fn add_if_new(
    seen: &mut HashMap<Turnstile, State>,
    paths: &mut BinaryHeap<Reverse<Trace>>,
    trace: Trace,
) {
    match seen.entry(trace.current.clone()) {
        Entry::Vacant(unseen) => {
            let _ = unseen.insert(State::Unknown);
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
    #[allow(unsafe_code)]
    #[allow(clippy::print_stdout)] // FIXME: remove `println!`s
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
            println!("Testing {path}");
            for expansion in Rc::new(path).expand() {
                match expansion {
                    Expansion::Qed(history) => {
                        match cache_proof(&mut seen, &paused, &history, true, &original) {
                            None => return Ok(()),
                            Some(proven) => {
                                for split in proven {
                                    let _ = paused.remove(&split);
                                }
                            }
                        }
                    }
                    Expansion::Contradiction(history) => {
                        match cache_proof(&mut seen, &paused, &history, false, &original) {
                            None => return Ok(()),
                            Some(proven) => {
                                for split in proven {
                                    let _ = paused.remove(&split);
                                }
                            }
                        }
                    }
                    Expansion::Linear(trace) => add_if_new(&mut seen, &mut paths, trace),
                    Expansion::Binary(split) => {
                        let Split {
                            ref lhs, ref rhs, ..
                        } = split;
                        add_if_new(&mut seen, &mut paths, lhs.clone());
                        add_if_new(&mut seen, &mut paths, rhs.clone());
                        // SAFETY: Added to `seen` above.
                        match unsafe {
                            (
                                seen.get(&split.lhs.current).unwrap_unchecked(),
                                seen.get(&split.rhs.current).unwrap_unchecked(),
                            )
                        } {
                            (&State::True, &State::True) => {
                                let (sl, sr) = split.sort();
                                println!("    Proved [ {sl}   {sr} ]");
                                match cache_proof(
                                    &mut seen,
                                    &paused,
                                    &split.history,
                                    true,
                                    &original,
                                ) {
                                    None => return Ok(()),
                                    Some(proven) => {
                                        for proven_split in proven {
                                            let _ = paused.remove(&proven_split);
                                        }
                                    }
                                }
                            }
                            (&State::False, _) | (_, &State::False) => {
                                let (sl, sr) = split.sort();
                                println!("    Disproved [ {sl}   {sr} ]");
                                match cache_proof(
                                    &mut seen,
                                    &paused,
                                    &split.history,
                                    false,
                                    &original,
                                ) {
                                    None => return Ok(()),
                                    Some(proven) => {
                                        for proven_split in proven {
                                            let _ = paused.remove(&proven_split);
                                        }
                                    }
                                }
                            }
                            _ => { /* fall through */ }
                        }
                        println!("    Pausing {split}");
                        let _ = paused.insert(split);
                    }
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
    pub(crate) fn expand(self: &Rc<Self>) -> Vec<Expansion> {
        if self.current.rhs.contains(&Ast::Top) {
            return vec![Expansion::Qed(Rc::clone(self))];
        }
        match self.current.rhs.only() {
            Some(&Ast::One) => return vec![Expansion::Qed(Rc::clone(self))],
            Some(&Ast::Bottom) => return vec![Expansion::Contradiction(Rc::clone(self))],
            Some(_) | None => {}
        }
        match self.current.rhs.pair() {
            Some((&Ast::Dual(ref neg), pos) | (pos, &Ast::Dual(ref neg))) => {
                if neg.as_ref() == pos {
                    return vec![Expansion::Qed(Rc::clone(self))];
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

    /// Continue with a single turnstile above the inference line.
    #[inline(always)]
    pub fn one(self: &Rc<Self>, child: Multiset<Ast>) -> Expansion {
        Expansion::Linear(Self {
            current: Turnstile { rhs: child },
            history: Some(Rc::clone(self)),
        })
    }

    /// Continue with two children.
    #[inline(always)]
    pub fn two(self: &Rc<Self>, lhs: Multiset<Ast>, rhs: Multiset<Ast>) -> Expansion {
        Expansion::Binary(Split {
            lhs: Self {
                current: Turnstile { rhs: lhs },
                history: None,
            },
            rhs: Self {
                current: Turnstile { rhs },
                history: None,
            },
            history: Rc::clone(self),
        })
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
    ) -> impl IntoIterator<Item = Expansion> {
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
        }
    }
}
