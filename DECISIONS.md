# tbf — Design decisions

Running log. Each entry: date, the decision, who proposed it, and who chose it. Newest first.
Nothing here is final until the owner says so; open questions live at the bottom.

## Decided

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

- The directive's exact grammar: where it may stand, and what else it may set.
- The tape's size, and where the pointer starts.
- The `eof` directive's spellings for 0, unchanged and −1 (no `-` allowed).
- The IR and its folds (runs, `[-]`, multiply loops, scans).
- When a loop is hot: the thresholds for each tier.
- The benchmark: which programs, and which other implementations tbf is measured against.
- A license. The repo is public with none yet, which means all rights reserved.
