# tbf — Design decisions

Running log. Each entry: date, the decision, who proposed it, and who chose it. Newest first.
Nothing here is final until the owner says so; open questions live at the bottom.

## Decided

### 2026-10-29 — A `tbf` line holding a command is an error, not code (owner: (a))

Found while testing: `tbf eof -1` holds a `-`, so under the header rule it was the first line of code; it ran, and the
end of input stayed 0, with no error. Offered (a) refuse a `tbf` line holding a command, (b) leave it as code, (c) take
any `tbf` line as a directive, commands and all (other tools would still run the `-`); design claude recommended (a).
The owner: "(a)". So **the line that ends the header is refused when its first word is `tbf`**:
`error: a directive cannot hold "-"; for −1 write tbf eof max`. A `tbf` line after the header is still a comment.

### 2026-10-29 — The benchmark: five classic programs against tritium, bff4 and bfc, timing whole runs (owner: (a))

Offered (a) `mandelbrot.b`, `hanoi.b`, `factor.b`, `long.b` and `dbfi.b` against tritium (an optimizing interpreter
with JIT and C backends), bff4 (an optimizing interpreter) and bfc (an LLVM ahead-of-time compiler in Rust), timing the
whole run, compile included; (b) the same, run time only; (c) another set. Design claude recommended (a): tbf is a JIT,
so its compile time is part of what a user waits for. The owner: "(a)". So **each tool is timed from start to finish on
the same machine; bfc is shown both with and without its compile step**. Each download is asked first, naming its
source and size.

### 2026-10-29 — The IR's first folds are seven; deeper analyses are exploring (owner: (c))

Design claude listed seven known wins: (1) runs, `+++++` as add 5, `>>>` as move 3; (2) clears, `[-]` as set 0; (3)
offsets, `>+>++<<` as adds at +1 and +2 with no move; (4) multiply and copy loops, `[->+>+++<<]` as cell[+1] += cell,
cell[+2] += 3 × cell, cell = 0, with no loop; (5) scans, `[>]` and `[<<]` as a search for the next zero, a memory search
for bytes; (6) known-zero loops, a loop at the start or right after another loop, dropped; (7) set then add, `[-]+++` as
set 3. Offered (a) all seven, (b) 1 and 2 first, the rest one by one against benchmarks, (c) all seven, deeper ones
later as exploring; design claude recommended (a). The owner: "(c)". So **the IR does the seven before any tier sees a
program**, and **deeper analyses, such as a loop that always runs a fixed number of times, are exploring (not built)**.

### 2026-10-29 — Code under Apache-2.0 WITH LLVM-exception, text under CC BY 4.0, as in CyborgPL (owner: (a))

Design claude offered Apache-2.0, MIT or Apache-2.0, MIT or GPL-3.0, saying CyborgPL used Apache-2.0. The owner:
"CyborgPL uses Apache 2.0 with LLVM exception, and CC-BY 4.0 for their text stuff (say README)" (GitHub's licence label
drops the exception). The LLVM exception fits a compiler: what a program picks up from the compiler while being compiled
owes nothing. Offered (a) the same split, (b) Apache-2.0 WITH LLVM-exception for everything, recommending (a); the
owner: "(a)". So **`LICENSE` is Apache-2.0 WITH LLVM-exception for the code, and `LICENSE-CC-BY-4.0` covers the
Markdown text**, credited as `tbf GitHub repo text: https://github.com/Artificial-IntelligenceAI/tbf`.

### 2026-10-29 — The tape is 2³⁰ cells; each guard is at least as large as the program's biggest single move (owner: (a))

The tape is reserved address space, backed only where touched, so its size costs no speed. Offered (a) 2³⁰ cells,
(b) 30,000, (c) a `tbf tape` directive, (a) by default, recommending (a). The owner: "(a)". So **the tape is 2³⁰ cells**
(1 GiB of address space for `u8`, 4 GiB for `u32`). Since the IR folds moves (`>>>>…` is one move by N), one move could
leap a one-page guard; so **each guard region is rounded up to cover the largest move or offset in the program**, which
tbf knows before it runs.

### 2026-10-29 — The pointer starts at cell 0, the tape's left end (owner: (a))

Offered (a) cell 0, a `<` there hitting the left guard, (b) the middle of the tape, recommending (a), the standard that
nearly every program assumes. The owner: "(a)". So **the pointer starts at cell 0**, and a `<` there stops with
`error: pointer moved left of cell 0`.

### 2026-10-29 — An unknown or repeated directive is an error; errors stop, and say "error" (owner: "Errors just error, and say \"error\"")

Offered (a) refuse to run, listing the known directives, (b) warn and run, (c) ignore it as a comment; design claude
recommended (a), since a typo like `tbf cel u16` ignored would run with 8-bit cells and print wrong output with no clue
why. The owner: "Errors just error, and say \"error\"". So **an unknown directive or one given twice stops tbf before
the program runs**, and an error's message begins `error:`, as in `error: unknown directive "cel"; known: cell, eof`.

### 2026-10-29 — Directives stand only at the top, before the first command, one per line (owner: (a))

The risk: another program's prose ("tested with tbf cell u16 and …") read as a setting. Offered (a) only before the
first command character, one per line, (b) anywhere, (c) the first line only, every setting together; design claude
recommended (a). The owner: "(a)". So **tbf reads directives from the lines before the first of the 8 commands, and stops
looking there**; a `tbf …` later in the file is a comment.

### 2026-10-29 — The `eof` directive's words: `tbf eof 0`, `tbf eof keep`, `tbf eof max` (owner: (a))

Offered (a) `0` / `keep` / `max`, (b) `zero` / `same` / `ones`, (c) `0` / `keep` / `255`; design claude recommended (a):
`max` is every bit set, −1 at any width (255 for `u8`, 65535 for `u16`), so one word covers each cell width, and none
of them needs a `-`. The owner: "(a)". So **`tbf eof 0`, `tbf eof keep`, `tbf eof max`**.

### 2026-10-29 — A program chooses what `,` does at the end of input with a directive, `tbf eof …`; without one, 0 (owner: (d))

Brainfuck leaves this undefined; tools set the cell to 0, to −1, or leave it unchanged, and a program's end-of-input
test depends on which (`,[.,]` stops only if the end sets 0). The owner: "what?"; design claude explained it again with
`,[.,]` and offered (a) 0, (b) unchanged, (c) −1, (d) a directive, (a) without one, recommending (a). The owner: "(d)".
So **a program may say what the end of input does, and gets 0 without a directive**. The spellings are open: `-1` cannot
be written as is, since `-` is a command.

### 2026-10-29 — The tape is one large reservation fenced by guard pages; compiled code checks no bounds (owner: (a))

For speed, design claude offered (a) a large fixed tape whose neighbouring pages are marked no-access, (b) 30,000 cells
with bounds checks, (c) a tape that grows to the right, recommending (a). The owner: "(a)". So **the tape is reserved
as address space up front (the OS backs only the pages a program touches), a no-access guard page sits past each end,
and no tier emits a bounds check**: running off an end traps, and tbf catches the trap and reports it as a Brainfuck
error, "pointer moved left of cell 0", instead of a crash. The cost is handling the OS's fault signal safely, per
platform.

### 2026-10-29 — tbf aims to be among the fastest Brainfuck implementations (owner)

The owner: "this Brainfuck should have at least one of the best perf compared to other BFs". So **speed is a goal, not
a side effect**: a design choice that costs speed has to say why, and the claim is checked by benchmarks against other
fast implementations on the same programs and the same machine.

### 2026-10-29 — A program names its cell width in a comment directive, `tbf cell u16`; without one, 8-bit wrapping (owner: (a))

Design claude offered 8-bit wrapping, a choice of 8, 16 or 32 bits, or 32-bit only. The owner: "can't you just pick in
the syntax? Like, how Rust does `i32`", then "Oh...": Brainfuck's grammar is its 8 commands, and everything else is a
comment. Design claude offered a directive inside a comment, `tbf cell u16`, which every other Brainfuck tool skips, so
the program stays valid Brainfuck; a directive may use none of the 8 command characters (`tbf-cell` would run a `-`).
Offered (a) the directive, 8-bit by default, (b) a flag only, (c) both, the flag winning, (d) 8-bit only; design claude
recommended (a), and the owner: "(a)". So **a program may name its width, `u8`, `u16` or `u32`, and gets `u8`, wrapping,
without one**. The width is known before anything runs, so each tier compiles for that one width.

### 2026-10-29 — Public, at `Artificial-IntelligenceAI/tbf` (owner: (c), "pub")

Design claude offered (a) `Soclets/tbf`, private, (b) `Soclets/tbf`, public, (c) the owner's own account, (d)
local git only, recommending (a). The owner: "(c)", "pub". So the repo is **public at
`github.com/Artificial-IntelligenceAI/tbf`**.

### 2026-10-29 — The name is `tbf`, Tankun's Brainfuck (owner: (a))

Design claude offered (a) `tbf`, (b) `tankun-bf`, (c) `hotbf`; the owner: "(a)". So the repo, the crate and the
command are **`tbf`**: `tbf run hello.bf`.

### 2026-10-29 — Written in Rust (owner: (a))

Design claude offered (a) Rust, (b) C++, (c) C: Cranelift is a Rust library with no stable C or C++ interface, so
(b) and (c) would need a Rust bridge anyway; LLVM is reached from Rust through `inkwell` or `llvm-sys`. Design claude
recommended (a); the owner: "(a)". The first machine is arm64 macOS, with Rust 1.96 and LLVM 23 from Homebrew.

### 2026-10-29 — Three tiers: interpreter → Cranelift → LLVM (owner: (a))

Design claude offered (a) interpreter → Cranelift → LLVM, (b) Cranelift → LLVM, compiling the whole program up
front, (c) interpreter → LLVM, recommending (a). The owner: "By 'compile everything up front', when? Precisely": in
(b), after the frontend and before the first instruction runs, on every run, every loop including ones that never run;
nothing is saved. The owner: "(a)". So **the interpreter starts at once and counts each loop; a hot loop goes to
Cranelift, a very hot one to LLVM on a background thread**. The interpreter is also the reference both compilers are
tested against.

### 2026-10-29 — Cranelift and LLVM both, tiered like HotSpot's C1 and C2 (owner: "What if we use both? Like Java's C1 C2")

Design claude offered Cranelift, LLVM, Wasm or JVM bytecode. The owner: "what would actually win in perf? (a) or
(b)?" Design claude: LLVM's code is faster, but on Brainfuck most of the speed comes from the IR's own folds (runs,
`[-]`, multiply loops, scans), which shrinks the gap, and LLVM costs far more compile time on every run; it recommended
Cranelift first, then LLVM added and benchmarked. The owner: "What if we use both? Like Java's C1 C2". So **Cranelift
is the quick tier (C1) and LLVM the optimizing tier (C2)**. Brainfuck makes a tier switch cheap: at a loop's entry the
whole state is the tape and the pointer, so a hot loop compiles to `fn(tape, ptr) -> ptr` and a tier-up is a pointer
swap at its next entry. There is no speculation to undo.

### 2026-10-29 — A JIT on an existing code generator, "write once, run anywhere" (owner)

Design claude offered AOT, JIT, or both through one IR, recommending AOT through C first. The owner: "Let's do a
'write once, run anywhere' style Brainfuck. So, a JIT with an already existing backend?" So **`tbf` compiles in memory
and runs at once, and the machine code comes from an existing code generator**, which brings every CPU it supports;
tbf writes the frontend, the IR and a thin lowering into that generator. No file is written.

### 2026-10-29 — tbf is a Brainfuck compiler; "frontend/backend" are a compiler's halves (owner: "By frontend/backend, I meant those of a compiler's")

Asked for "a frontend/backend for Brainfuck", design claude read it as a web app (a page and a server) and offered
stacks. The owner: "What is a server for?", then "By frontend/backend, I meant those of a compiler's". So **the
frontend reads Brainfuck source into an IR and optimizes it; a backend turns the IR into something that runs**. There
is no web page and no server.

## Open

- Deeper IR analyses (exploring): fixed trip counts, and others found by benchmark.
- When a loop is hot: the thresholds for each tier.
- What `.` writes from a `u16` or `u32` cell: built as its low byte for now.
