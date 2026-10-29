# tbf

Tankun's Brainfuck: a tiered JIT for Brainfuck. One frontend reads source into an IR; an interpreter runs it at
once, hot loops are compiled by Cranelift, and the hottest by LLVM, like HotSpot's C1 and C2.

Status: tier 0 runs. `cargo build --release`, then `target/release/tbf run examples/hello.bf`. The JIT tiers and the
guard-paged tape are next; the decisions so far are in [DECISIONS.md](DECISIONS.md).

## Licences

- [`LICENSE`](LICENSE): Apache-2.0 WITH LLVM-exception, for the code and everything not listed below. Programs you
  run or compile with tbf owe nothing; a fork keeps the notices and may not use the name.
- [`LICENSE-CC-BY-4.0`](LICENSE-CC-BY-4.0): CC BY 4.0, for this repo's text (its Markdown files: this README,
  `DECISIONS.md` and any notes): copy, translate, remix and use it anywhere, with credit. Credit it exactly as
  `tbf GitHub repo text: https://github.com/Artificial-IntelligenceAI/tbf`, and, as the licence asks, say it is under
  CC BY 4.0 and whether you changed it.
