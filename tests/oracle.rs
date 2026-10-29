//! The folds against the source: a plain interpreter that reads the bytes one at a time, sharing nothing with
//! tbf's parser, IR or interpreter, runs thousands of random programs beside tbf, folded and unfolded.

use tbf::interp::{Limit, Stop};
use tbf::settings::{Eof, Settings, Width};

const TAPE: usize = 4096;
const FUEL: u64 = 20_000;

#[derive(Debug, PartialEq)]
enum End {
    Done,
    OffTape,
    Fuel,
}

/// One command at a time; every command that reads or writes a cell checks it is on the tape.
fn plain(src: &[u8], settings: Settings, input: &[u8]) -> (Vec<u8>, End) {
    let mask = settings.cell.mask();
    let mut jump = vec![0; src.len()];
    let mut stack = Vec::new();
    for (i, &b) in src.iter().enumerate() {
        match b {
            b'[' => stack.push(i),
            b']' => {
                let j = stack.pop().unwrap();
                jump[i] = j;
                jump[j] = i;
            }
            _ => {}
        }
    }
    let mut tape = vec![0u32; TAPE];
    let (mut ptr, mut pc, mut fuel, mut inp) = (0i64, 0, FUEL, input.iter());
    let mut out = Vec::new();
    while pc < src.len() {
        let b = src[pc];
        if matches!(b, b'+' | b'-' | b'.' | b',' | b'[' | b']') && !(0..TAPE as i64).contains(&ptr) {
            return (out, End::OffTape);
        }
        let c = || ptr as usize;
        match b {
            b'+' => tape[c()] = tape[c()].wrapping_add(1) & mask,
            b'-' => tape[c()] = tape[c()].wrapping_sub(1) & mask,
            b'>' => ptr += 1,
            b'<' => ptr -= 1,
            b'.' => out.push(tape[c()] as u8),
            b',' => match inp.next() {
                Some(&v) => tape[c()] = v as u32,
                None => match settings.eof {
                    Eof::Zero => tape[c()] = 0,
                    Eof::Keep => {}
                    Eof::Max => tape[c()] = mask,
                },
            },
            b'[' if tape[c()] == 0 => pc = jump[pc],
            b']' if tape[c()] != 0 => {
                if fuel == 0 {
                    return (out, End::Fuel);
                }
                fuel -= 1;
                pc = jump[pc];
            }
            _ => {}
        }
        pc += 1;
    }
    (out, End::Done)
}

fn tbf(src: &[u8], fold: bool, input: &[u8]) -> (Vec<u8>, Result<(), Stop>) {
    let mut out = Vec::new();
    let r = tbf::run(src, fold, TAPE, input, &mut out, &mut Limit(FUEL));
    (out, r)
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// Short programs heavy in what the folds look for.
fn program(rng: &mut Rng, depth: u32, out: &mut String) {
    const SNIPPETS: &[&str] = &[
        "[-]", "[+]", "[->+<]", "[->>+++<<]", "[-<+>]", "[+>--<]", "[->+>-<<]", "[>]", "[<]", "[>>]", "[<<<]",
        "[-]+++", "+++++", "-----", ">>>", "<<",
    ];
    for _ in 0..rng.below(12) + 1 {
        match rng.below(14) {
            0..=1 => out.push('+'),
            2 => out.push('-'),
            3..=4 => out.push('>'),
            5 => out.push('<'),
            6 => out.push('.'),
            7 => out.push(','),
            8..=10 => out.push_str(SNIPPETS[rng.below(SNIPPETS.len() as u64) as usize]),
            _ if depth < 3 => {
                out.push('[');
                program(rng, depth + 1, out);
                out.push(']');
            }
            _ => out.push('.'),
        }
    }
}

#[test]
fn folds_match_the_source() {
    let mut rng = Rng(0x7bf_2026_1029);
    let (mut compared, mut skipped) = (0, 0);
    for case in 0..20_000 {
        let cell = [("u8", Width::U8), ("u16", Width::U16), ("u32", Width::U32)][case % 3];
        let eof = [("0", Eof::Zero), ("keep", Eof::Keep), ("max", Eof::Max)][(case / 3) % 3];
        let mut src = format!("tbf cell {}\ntbf eof {}\n>>>>>>>>", cell.0, eof.0);
        program(&mut rng, 0, &mut src);
        src.push('.');
        let input: Vec<u8> = (0..rng.below(4)).map(|_| rng.below(256) as u8).collect();
        let settings = Settings { cell: cell.1, eof: eof.1 };

        let (want, end) = plain(src.as_bytes(), settings, &input);
        if end != End::Done {
            // Off the tape, a fold may drop a touch (`+-`) and so not stop; out of fuel, nothing to compare.
            skipped += 1;
            continue;
        }
        for fold in [false, true] {
            let (got, r) = tbf(src.as_bytes(), fold, &input);
            assert_eq!(r, Ok(()), "case {case}, fold {fold}: {src:?}");
            assert_eq!(got, want, "case {case}, fold {fold}: {src:?}");
        }
        compared += 1;
    }
    assert!(compared > 10_000, "only {compared} compared, {skipped} skipped");
}
