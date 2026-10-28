# tbf — Design decisions

Running log. Each entry: date, the decision, who proposed it, and who chose it. Newest first.
Nothing here is final until the owner says so; open questions live at the bottom.

## Decided

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
