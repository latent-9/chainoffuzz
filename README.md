# ChainOfFuzz

Robustness is not proven by what works. It is proven by everything that failed trying to break it.

ChainOfFuzz is a mutation fuzzer for consensus adjacent parsers. One engine, many targets. Every target carries its own receipt: iterations run, inputs accepted, inputs rejected, panics caught.

The philosophy is simple. A parser that guards consensus must survive an adversary that never gets tired. Coverage guided fuzzers are smart. This one is not. It is patient, it is reproducible, and it keeps receipts.

## Receipts

A receipt is only valid if you can rerun it yourself.

| target | crate | iterations | accepted | rejected | panics |
|---|---|---|---|---|---|
| sbpf-elf | solana-sbpf 0.25.0 | 100,000,000 | 65,110 | 99,934,890 | 0 |
| block-component | solana-entry 4.5.0-alpha.2 | 10,000,000 | 548,534 | 9,451,466 | 0 |
| votor-wire | agave-votor-messages 4.5.0-alpha.2 | 10,000,000 | 256,597 | 9,743,403 | 0 |

Rerun: `cargo run --release -- <target> <iterations>`, default seed 12648430. The sbpf-elf campaign was run twice independently and produced identical counts, digit for digit.

## Usage

```bash
cargo run --release -- sbpf-elf 10000000
```

Findings land in `findings/` as the exact input that produced a panic, named by iteration. A finding is a bug until proven otherwise. Zero findings is also a result, as long as the receipt says how hard you looked.

## Targets

- `sbpf-elf`: ELF parse and verify path of solana-sbpf, the parser behind on chain program deployment. Seeds are real SPL program binaries in `seeds/sbpf-elf/` plus synthetic minimal ELFs for every sbpf version.
- `block-component`: wincode deserialization of block markers, the wire format carrying block footers, headers and update-parent markers between leader and replay. Seeds are synthetic valid markers for every variant.
- `votor-wire`: the hand written unsafe deserialization of consensus votes and certificates, the first gate for all network input into votor. Seeds are real signed vote messages.

## Adding a target

Implement the `Target` trait: a name, a description, a seed corpus, and a run function. The engine, the mutator, and the oracle are shared. Register it in `targets/mod.rs` and the CLI picks it up.

## Rules of the house

- A panic is a finding. A clean rejection is not.
- Every claim carries a receipt. No receipt, no claim.
- Seeds come from the real world. Synthetic seeds only open the door.
- Everything here runs locally against open source code. Findings in production parsers go to the project security policy first, never straight to the public findings folder.
