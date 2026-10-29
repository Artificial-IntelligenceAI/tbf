//! Whole programs, through `tbf::run`.

use tbf::interp::{Stop, Unlimited};

fn run(src: &str, input: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    match tbf::run(src.as_bytes(), true, 1 << 16, input, &mut out, &mut Unlimited) {
        Ok(()) => Ok(out),
        Err(Stop::Error(e)) => Err(e.render("t.bf")),
        Err(Stop::Fuel) => unreachable!(),
    }
}

#[test]
fn hello() {
    let src = std::fs::read_to_string("examples/hello.bf").unwrap();
    assert_eq!(run(&src, b"").unwrap(), b"Hello, World!\n");
}

#[test]
fn cat() {
    let src = std::fs::read_to_string("examples/cat.bf").unwrap();
    assert_eq!(run(&src, b"abc\n").unwrap(), b"abc\n");
}

#[test]
fn end_of_input() {
    // The cell holds 7, then `,` meets the end of input.
    assert_eq!(run("+++++++,.", b"").unwrap(), [0]);
    assert_eq!(run("tbf eof 0\n+++++++,.", b"").unwrap(), [0]);
    assert_eq!(run("tbf eof keep\n+++++++,.", b"").unwrap(), [7]);
    assert_eq!(run("tbf eof max\n+++++++,.", b"").unwrap(), [255]);
    assert_eq!(run("tbf cell u16\ntbf eof max\n,>++++++++[<+>-]<.", b"").unwrap(), [7]);
}

#[test]
fn widths_wrap() {
    // 256: 0 in a u8, not in a u16; 65536: 0 in a u16, not in a u32.
    let at256 = "++++++++++++++++[>++++++++++++++++<-]>[[-]<+>]<.";
    assert_eq!(run(at256, b"").unwrap(), [0]);
    assert_eq!(run(&format!("tbf cell u16\n{at256}"), b"").unwrap(), [1]);
    let at65536 = "++++++++++++++++[>++++++++++++++++<-]>[<++++++++++++++++>-]<[>++++++++++++++++<-]>[[-]<+>]<.";
    assert_eq!(run(&format!("tbf cell u16\n{at65536}"), b"").unwrap(), [0]);
    assert_eq!(run(&format!("tbf cell u32\n{at65536}"), b"").unwrap(), [1]);
    assert_eq!(run("-.", b"").unwrap(), [255]);
}

#[test]
fn errors() {
    assert_eq!(run("<+", b""), Err("error: pointer moved left of cell 0".into()));
    assert_eq!(run("+[>+]", b""), Err("error: pointer moved right of cell 65535".into()));
    assert_eq!(run("+[<]", b""), Err("error: pointer moved left of cell 0".into()));
    assert_eq!(run("+[-<+>]", b""), Err("error: pointer moved left of cell 0".into()));
    assert_eq!(run("tbf eof neg\n.", b""), Err("error: unknown eof \"neg\"; known: 0, keep, max\n  at t.bf:1:9".into()));
    assert_eq!(run("]", b""), Err("error: this ] has no [\n  at t.bf:1:1".into()));
}

#[test]
fn output_before_an_error_is_kept() {
    assert_eq!(run("+.<+", b""), Err("error: pointer moved left of cell 0".into()));
    let mut out = Vec::new();
    let _ = tbf::run(b"+.<+", true, 16, &b""[..], &mut out, &mut Unlimited);
    assert_eq!(out, [1]);
}

#[test]
fn a_multiply_loop_on_zero_touches_nothing() {
    // At cell 0 holding 0, `[-<+>]` never runs, so cell −1 is never touched.
    assert_eq!(run("[-<+>]+.", b"").unwrap(), [1]);
    assert_eq!(run(",[-<+>]+.", b"\0").unwrap(), [1]);
}
