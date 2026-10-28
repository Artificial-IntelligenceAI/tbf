# tbf — Design decisions

Running log. Each entry: date, the decision, who proposed it, and who chose it. Newest first.
Nothing here is final until the owner says so; open questions live at the bottom.

## Decided

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
