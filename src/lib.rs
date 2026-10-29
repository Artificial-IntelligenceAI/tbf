//! tbf: Tankun's Brainfuck.

pub mod cell;
pub mod directives;
pub mod error;
pub mod interp;
pub mod ir;
pub mod parse;
pub mod settings;

use std::io::{Read, Write};

use cell::Cell;
use interp::{Fuel, Stop};
use settings::Width;

/// The tape: 2³⁰ cells.
pub const TAPE_LEN: usize = 1 << 30;

/// Reads, folds and runs a program. `fold: false` runs the unfolded IR, the reference for tests.
pub fn run<R: Read, W: Write, F: Fuel>(
    src: &[u8],
    fold: bool,
    tape_len: usize,
    input: R,
    output: W,
    fuel: &mut F,
) -> Result<(), Stop> {
    let settings = directives::read(src).map_err(Stop::Error)?;
    let nodes = parse::parse(src).map_err(Stop::Error)?;
    let ops = if fold { ir::lower(&nodes, settings.cell) } else { ir::lower_plain(&nodes, settings.cell) };
    let prog = interp::compile(&ops);
    fn go<C: Cell, R: Read, W: Write, F: Fuel>(
        prog: &interp::Program,
        eof: settings::Eof,
        tape_len: usize,
        input: R,
        output: W,
        fuel: &mut F,
    ) -> Result<(), Stop> {
        let mut tape = vec![C::default(); tape_len];
        interp::run(prog, eof, &mut tape, input, output, fuel)
    }
    match settings.cell {
        Width::U8 => go::<u8, _, _, _>(&prog, settings.eof, tape_len, input, output, fuel),
        Width::U16 => go::<u16, _, _, _>(&prog, settings.eof, tape_len, input, output, fuel),
        Width::U32 => go::<u32, _, _, _>(&prog, settings.eof, tape_len, input, output, fuel),
    }
}
