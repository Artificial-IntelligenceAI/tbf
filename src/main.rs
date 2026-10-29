use std::io::{self, BufWriter};
use std::process::ExitCode;

use tbf::interp::{Stop, Unlimited};

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let [cmd, path] = args.as_slice() else { return fail("usage: tbf run <file.bf>") };
    if cmd != "run" {
        return fail("usage: tbf run <file.bf>");
    }
    let name = path.to_string_lossy();
    let src = match std::fs::read(path) {
        Ok(src) => src,
        Err(e) => return fail(&format!("cannot read {name}: {e}")),
    };
    let input = io::stdin().lock();
    let output = BufWriter::with_capacity(1 << 16, io::stdout().lock());
    match tbf::run(&src, true, tbf::TAPE_LEN, input, output, &mut Unlimited) {
        Ok(()) => ExitCode::SUCCESS,
        Err(Stop::Error(e)) => {
            eprintln!("{}", e.render(&name));
            ExitCode::FAILURE
        }
        Err(Stop::Fuel) => unreachable!("an unlimited run has no fuel to run out of"),
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("error: {message}");
    ExitCode::FAILURE
}
