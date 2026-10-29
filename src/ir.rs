//! The IR every tier runs, and the folds that build it.
//!
//! Ops address cells by offset from the pointer, so a block moves the pointer once, before a loop or at its end.
//! Folds: (1) runs, in the parser; (2) clears; (3) offsets; (4) multiply and copy loops; (5) scans;
//! (6) known-zero loops; (7) set then add.
//!
//! An op touches only the cells the source would have touched, so a program that stays on the tape behaves the
//! same. A fold may drop a touch (`+-`), never add one: a multiply loop does nothing when its cell is 0.

use crate::parse::Node;
use crate::settings::Width;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    /// `cell[off] += v`, wrapping.
    Add { off: i64, v: u32 },
    /// `cell[off] = v`.
    Set { off: i64, v: u32 },
    /// If `cell[src]` is not 0: `cell[src + o] += cell[src] * k` for each `(o, k)`, then `cell[src] = 0`.
    Mul { src: i64, targets: Vec<(i64, u32)> },
    Move(i64),
    Out { off: i64 },
    In { off: i64 },
    /// `while cell[0] != 0 { ptr += stride }`
    Scan(i64),
    /// `while cell[0] != 0 { body }`
    Loop(Vec<Op>),
}

impl Op {
    /// Whether this op reads or writes `cell[off]`. Ops that move the pointer or run a body touch everything.
    fn touches(&self, off: i64) -> bool {
        match self {
            Op::Add { off: o, .. } | Op::Set { off: o, .. } | Op::Out { off: o } | Op::In { off: o } => *o == off,
            Op::Mul { src, targets } => *src == off || targets.iter().any(|(o, _)| src + o == off),
            Op::Move(_) | Op::Scan(_) | Op::Loop(_) => true,
        }
    }
}

/// Builds the IR, every fold applied.
pub fn lower(nodes: &[Node], width: Width) -> Vec<Op> {
    let mut ops = block(nodes, width);
    if let Some(Op::Move(_)) = ops.last() {
        ops.pop();
    }
    known_zero(&mut ops, true);
    ops
}

/// Builds the IR with no folds past the parser's runs: the reference the folds are tested against.
pub fn lower_plain(nodes: &[Node], width: Width) -> Vec<Op> {
    nodes
        .iter()
        .map(|n| match n {
            Node::Add(d) => Op::Add { off: 0, v: width.wrap(*d) },
            Node::Move(d) => Op::Move(*d),
            Node::Out => Op::Out { off: 0 },
            Node::In => Op::In { off: 0 },
            Node::Loop(body) => Op::Loop(lower_plain(body, width)),
        })
        .collect()
}

/// One block, at offsets from its start; the pointer moves only before a loop and at the end.
fn block(nodes: &[Node], width: Width) -> Vec<Op> {
    let mut out = Vec::new();
    let mut cur = 0i64;
    for n in nodes {
        match n {
            Node::Add(d) => push(&mut out, Op::Add { off: cur, v: width.wrap(*d) }, width),
            Node::Move(d) => cur += d,
            Node::Out => out.push(Op::Out { off: cur }),
            Node::In => out.push(Op::In { off: cur }),
            Node::Loop(body) => {
                if let Some(op) = flat_loop(body, width, cur) {
                    push(&mut out, op, width);
                } else {
                    if cur != 0 {
                        out.push(Op::Move(cur));
                        cur = 0;
                    }
                    match body.as_slice() {
                        [Node::Move(s)] => out.push(Op::Scan(*s)),
                        _ => out.push(Op::Loop(block(body, width))),
                    }
                }
            }
        }
    }
    if cur != 0 {
        out.push(Op::Move(cur));
    }
    out
}

/// Folds 2 and 4: a loop of adds and moves that returns to its cell and steps it by ±1 is a clear or a multiply.
fn flat_loop(body: &[Node], width: Width, cur: i64) -> Option<Op> {
    let mut deltas: Vec<(i64, i64)> = Vec::new();
    let mut pos = 0i64;
    for n in body {
        match n {
            Node::Add(d) => match deltas.iter_mut().find(|(o, _)| *o == pos) {
                Some((_, sum)) => *sum += d,
                None => deltas.push((pos, *d)),
            },
            Node::Move(d) => pos += d,
            _ => return None,
        }
    }
    if pos != 0 {
        return None;
    }
    let step = deltas.iter().find(|(o, _)| *o == 0).map_or(0, |&(_, d)| width.wrap(d));
    let mask = width.mask();
    // Stepping by −1, the loop runs cell times; by +1, (−cell) times, so each factor is negated.
    let negate = match step {
        s if s == mask => false,
        1 => true,
        _ => return None,
    };
    let mut targets: Vec<(i64, u32)> = deltas
        .into_iter()
        .filter(|&(o, _)| o != 0)
        .map(|(o, d)| (o, width.wrap(if negate { -d } else { d })))
        .filter(|&(_, k)| k != 0)
        .collect();
    if targets.is_empty() {
        return Some(Op::Set { off: cur, v: 0 });
    }
    targets.sort_unstable_by_key(|&(o, _)| o);
    Some(Op::Mul { src: cur, targets })
}

/// Folds 3 and 7: an add or set merges into an earlier add or set of the same cell, past ops on other cells.
fn push(out: &mut Vec<Op>, op: Op, width: Width) {
    let (off, v) = match op {
        Op::Add { off, v } | Op::Set { off, v } => (off, v),
        _ => return out.push(op),
    };
    let is_set = matches!(op, Op::Set { .. });
    for i in (0..out.len()).rev() {
        if !out[i].touches(off) {
            if matches!(out[i], Op::Add { .. } | Op::Set { .. } | Op::Mul { .. }) {
                continue;
            }
            break;
        }
        match (&mut out[i], is_set) {
            (Op::Add { .. } | Op::Set { .. }, true) => {
                out[i] = Op::Set { off, v };
                return;
            }
            (Op::Add { v: prev, .. }, false) => {
                *prev = prev.wrapping_add(v) & width.mask();
                if *prev == 0 {
                    out.remove(i);
                }
                return;
            }
            (Op::Set { v: prev, .. }, false) => {
                *prev = prev.wrapping_add(v) & width.mask();
                return;
            }
            _ => break,
        }
    }
    out.push(op);
}

/// Fold 6: a loop, scan, clear or multiply on a cell known to be 0 does nothing.
/// `clean` holds while no cell has been written, at the program's start: then every cell is 0, and an add is a set.
fn known_zero(ops: &mut Vec<Op>, mut clean: bool) {
    let mut zero_here = clean;
    let mut i = 0;
    while i < ops.len() {
        let keep = match &mut ops[i] {
            Op::Loop(_) | Op::Scan(_) if zero_here => false,
            Op::Set { v: 0, .. } | Op::Mul { .. } if clean => false,
            Op::Set { off: 0, v: 0 } | Op::Mul { src: 0, .. } if zero_here => false,
            Op::Loop(body) => {
                known_zero(body, false);
                zero_here = true;
                clean = false;
                true
            }
            Op::Scan(_) => {
                zero_here = true;
                true
            }
            Op::Move(_) => {
                zero_here = clean;
                true
            }
            Op::Out { .. } => true,
            op => {
                if clean && let Op::Add { off, v } = *op {
                    *op = Op::Set { off, v };
                }
                clean = false;
                let writes_here = match op {
                    Op::Add { off, .. } | Op::Set { off, .. } | Op::In { off } => *off == 0,
                    Op::Mul { src, targets } => *src == 0 || targets.iter().any(|(o, _)| *src + o == 0),
                    _ => false,
                };
                if writes_here {
                    zero_here = matches!(op, Op::Set { v: 0, .. } | Op::Mul { src: 0, .. });
                }
                true
            }
        };
        if keep {
            i += 1;
        } else {
            ops.remove(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;
    use Op::*;

    fn ir(src: &str) -> Vec<Op> {
        lower(&parse(src.as_bytes()).unwrap(), Width::U8)
    }

    #[test]
    fn offsets() {
        assert_eq!(ir(",>+>++<<."), vec![In { off: 0 }, Add { off: 1, v: 1 }, Add { off: 2, v: 2 }, Out { off: 0 }]);
        assert_eq!(ir(",+>+<+."), vec![In { off: 0 }, Add { off: 0, v: 2 }, Add { off: 1, v: 1 }, Out { off: 0 }]);
    }

    #[test]
    fn clears_and_set_then_add() {
        assert_eq!(ir(",[-]+++."), vec![In { off: 0 }, Set { off: 0, v: 3 }, Out { off: 0 }]);
        assert_eq!(ir(",[+]."), vec![In { off: 0 }, Set { off: 0, v: 0 }, Out { off: 0 }]);
        assert_eq!(ir(",+++[-]."), vec![In { off: 0 }, Set { off: 0, v: 0 }, Out { off: 0 }]);
    }

    #[test]
    fn multiply() {
        assert_eq!(
            ir(",[->+>+++<<]"),
            vec![In { off: 0 }, Mul { src: 0, targets: vec![(1, 1), (2, 3)] }]
        );
        // Stepping up by one runs (−cell) times: each factor negated.
        assert_eq!(ir(",[+>++<]"), vec![In { off: 0 }, Mul { src: 0, targets: vec![(1, 254)] }]);
        // Stepping by two is not folded.
        assert!(matches!(ir(",[-->+<]").as_slice(), [In { .. }, Loop(_)]));
    }

    #[test]
    fn scans() {
        assert_eq!(ir(",[>]."), vec![In { off: 0 }, Scan(1), Out { off: 0 }]);
        assert_eq!(ir(",>[<<]"), vec![In { off: 0 }, Move(1), Scan(-2)]);
    }

    #[test]
    fn known_zero_loops() {
        assert_eq!(ir("[.]>[-]>[->+<]+."), vec![Set { off: 2, v: 1 }, Out { off: 2 }]);
        assert_eq!(ir(",[.][.]"), vec![In { off: 0 }, Loop(vec![Out { off: 0 }])]);
        assert_eq!(ir(",[.,]>[.]"), vec![In { off: 0 }, Loop(vec![Out { off: 0 }, In { off: 0 }]), Move(1), Loop(vec![Out { off: 0 }])]);
    }
}
