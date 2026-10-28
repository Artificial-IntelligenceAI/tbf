# tbf — Design decisions

Running log. Each entry: date, the decision, who proposed it, and who chose it. Newest first.
Nothing here is final until the owner says so; open questions live at the bottom.

## Decided

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

- Cell width, tape length, and what `,` does at the end of input.
- The IR and its folds (runs, `[-]`, multiply loops, scans).
- When a loop is hot: the thresholds for each tier.
- A license. The repo is public with none yet, which means all rights reserved.
