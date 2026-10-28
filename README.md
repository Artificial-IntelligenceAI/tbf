# tbf

Tankun's Brainfuck: a tiered JIT for Brainfuck. One frontend reads source into an IR; an interpreter runs it at
once, hot loops are compiled by Cranelift, and the hottest by LLVM, like HotSpot's C1 and C2.

Status: design. Nothing is built yet; the decisions so far are in [DECISIONS.md](DECISIONS.md).
