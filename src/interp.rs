//! Tier 0: runs the IR at once, from a flat list of instructions.

use std::io::{ErrorKind, Read, Write};

use crate::cell::Cell;
use crate::error::Error;
use crate::ir::Op;
use crate::settings::Eof;

/// Why a run stopped early.
#[derive(Debug, PartialEq, Eq)]
pub enum Stop {
    Error(Error),
    /// Out of fuel: only when a run is given a limit, as tests do.
    Fuel,
}

/// Counts loop iterations; `Unlimited` compiles to nothing.
pub trait Fuel {
    fn tick(&mut self) -> bool;
}

pub struct Unlimited;

impl Fuel for Unlimited {
    #[inline(always)]
    fn tick(&mut self) -> bool {
        true
    }
}

pub struct Limit(pub u64);

impl Fuel for Limit {
    fn tick(&mut self) -> bool {
        if self.0 == 0 {
            return false;
        }
        self.0 -= 1;
        true
    }
}

#[derive(Clone, Copy, Debug)]
enum Ins {
    Add(i64, u32),
    Set(i64, u32),
    /// The source offset, then the range of its targets in `Program::targets`.
    Mul(i64, u32, u32),
    Move(i64),
    Out(i64),
    In(i64),
    Scan(i64),
    /// If the cell is 0, jump past the matching `Close`.
    Open(usize),
    /// If the cell is not 0, jump back past the matching `Open`.
    Close(usize),
}

pub struct Program {
    code: Vec<Ins>,
    targets: Vec<(i64, u32)>,
}

pub fn compile(ops: &[Op]) -> Program {
    let mut p = Program { code: Vec::new(), targets: Vec::new() };
    emit(&mut p, ops);
    p
}

fn emit(p: &mut Program, ops: &[Op]) {
    for op in ops {
        let ins = match op {
            Op::Add { off, v } => Ins::Add(*off, *v),
            Op::Set { off, v } => Ins::Set(*off, *v),
            Op::Mul { src, targets } => {
                let start = p.targets.len() as u32;
                p.targets.extend(targets);
                Ins::Mul(*src, start, p.targets.len() as u32)
            }
            Op::Move(d) => Ins::Move(*d),
            Op::Out { off } => Ins::Out(*off),
            Op::In { off } => Ins::In(*off),
            Op::Scan(s) => Ins::Scan(*s),
            Op::Loop(body) => {
                let open = p.code.len();
                p.code.push(Ins::Open(0));
                emit(p, body);
                p.code.push(Ins::Close(open + 1));
                p.code[open] = Ins::Open(p.code.len());
                continue;
            }
        };
        p.code.push(ins);
    }
}

#[inline(always)]
fn at(p: i64, len: usize) -> Result<usize, Stop> {
    if p < 0 {
        Err(Stop::Error(Error::new("pointer moved left of cell 0")))
    } else if p as u64 >= len as u64 {
        Err(Stop::Error(Error::new(format!("pointer moved right of cell {}", len - 1))))
    } else {
        Ok(p as usize)
    }
}

/// Runs `prog` on `tape`, from cell 0. Output written before an error is flushed before it is returned.
pub fn run<C: Cell, R: Read, W: Write, F: Fuel>(
    prog: &Program,
    eof: Eof,
    tape: &mut [C],
    mut input: R,
    mut out: W,
    fuel: &mut F,
) -> Result<(), Stop> {
    let result = exec(prog, eof, tape, &mut input, &mut out, fuel);
    let flushed = out.flush().map_err(write_error);
    result.and(flushed)
}

fn write_error(e: std::io::Error) -> Stop {
    Stop::Error(Error::new(format!("cannot write output: {e}")))
}

fn exec<C: Cell, R: Read, W: Write, F: Fuel>(
    prog: &Program,
    eof: Eof,
    tape: &mut [C],
    input: &mut R,
    out: &mut W,
    fuel: &mut F,
) -> Result<(), Stop> {
    let code = &prog.code[..];
    let len = tape.len();
    let zero = C::default();
    let mut ptr = 0i64;
    let mut pc = 0;
    while let Some(&ins) = code.get(pc) {
        pc += 1;
        match ins {
            Ins::Add(off, v) => {
                let c = &mut tape[at(ptr + off, len)?];
                *c = c.wrapping_add(C::from_u32(v));
            }
            Ins::Set(off, v) => tape[at(ptr + off, len)?] = C::from_u32(v),
            Ins::Mul(src, start, end) => {
                let s = at(ptr + src, len)?;
                let n = tape[s];
                if n != zero {
                    for &(o, k) in &prog.targets[start as usize..end as usize] {
                        let c = &mut tape[at(ptr + src + o, len)?];
                        *c = c.wrapping_add(n.wrapping_mul(C::from_u32(k)));
                    }
                    tape[s] = zero;
                }
            }
            Ins::Move(d) => ptr += d,
            Ins::Out(off) => {
                let byte = tape[at(ptr + off, len)?].to_u32() as u8;
                out.write_all(&[byte]).map_err(write_error)?;
            }
            Ins::In(off) => {
                let i = at(ptr + off, len)?;
                out.flush().map_err(write_error)?;
                if let Some(byte) = read_byte(input)? {
                    tape[i] = C::from_u32(byte as u32);
                } else {
                    match eof {
                        Eof::Zero => tape[i] = zero,
                        Eof::Keep => {}
                        Eof::Max => tape[i] = C::MAX,
                    }
                }
            }
            Ins::Scan(stride) => {
                let mut i = at(ptr, len)?;
                if stride == 1 {
                    match tape[i..].iter().position(|&c| c == zero) {
                        Some(n) => i += n,
                        None => return at(len as i64, len).map(|_| ()),
                    }
                } else {
                    while tape[i] != zero {
                        if !fuel.tick() {
                            return Err(Stop::Fuel);
                        }
                        i = at(i as i64 + stride, len)?;
                    }
                }
                ptr = i as i64;
            }
            Ins::Open(past) => {
                if tape[at(ptr, len)?] == zero {
                    pc = past;
                }
            }
            Ins::Close(back) => {
                if tape[at(ptr, len)?] != zero {
                    if !fuel.tick() {
                        return Err(Stop::Fuel);
                    }
                    pc = back;
                }
            }
        }
    }
    Ok(())
}

fn read_byte<R: Read>(input: &mut R) -> Result<Option<u8>, Stop> {
    let mut b = [0u8];
    loop {
        match input.read(&mut b) {
            Ok(0) => return Ok(None),
            Ok(_) => return Ok(Some(b[0])),
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(Stop::Error(Error::new(format!("cannot read input: {e}")))),
        }
    }
}
