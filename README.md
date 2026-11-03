# ChainOfFuzz

Robustness is not proven by what works. It is proven by everything that failed trying to break it.

ChainOfFuzz is a mutation fuzzer for consensus adjacent parsers. One engine, many targets. Every target carries its own receipt: iterations run, inputs accepted, inputs rejected, panics caught.

The philosophy is simple. A parser that guards consensus must survive an adversary that never gets tired. Coverage guided fuzzers are smart. This one is not. It is patient, it is reproducible, and it keeps receipts.

## Receipts

A receipt is only valid if you can rerun it yourself. All campaigns use the pinned crate versions below, one engine, seed 12648430.

| target | crate | iterations | accepted | rejected | panics |
|---|---|---|---|---|---|
| sbpf-elf | solana-sbpf 0.25.0 | 100,000,000 | 65,110 | 99,934,890 | 0 |
| block-component | solana-entry 4.5.0-alpha.2 | 10,000,000 | 447,763 | 9,552,237 | 0 |
| votor-wire | agave-votor-messages 4.5.0-alpha.2 | 10,000,000 | 256,594 | 9,743,406 | 0 |
| shred-ingress | solana-ledger 4.5.0-alpha.2 | 10,000,000 | 187,595 | 9,812,405 | 0 |
| **total** | | **130,000,000** | **957,062** | **129,042,938** | **0** |

Rerun: `cargo run --release -- <target> <iterations>`, default seed 12648430. Same seed, same engine, same counts.

The sbpf-elf row is the 2026-10-17 campaign; nothing that target runs against has changed since, so its counts remain reproducible. It was run twice independently at the time and produced identical counts, digit for digit. The other three targets were re-run 2026-11-03 after their seed corpora were fixed or extended.

## Usage

```bash
cargo run --release -- <target> <iterations>

# examples
cargo run --release -- sbpf-elf 100000000
cargo run --release -- votor-wire 10000000
```

Targets: `sbpf-elf`, `block-component`, `votor-wire`, `shred-ingress`. Run without arguments for the list.

Findings land in `findings/` as the exact input that produced a panic, named by iteration. A finding is a bug until proven otherwise. Zero findings is also a result, as long as the receipt says how hard you looked.

## Targets

- `sbpf-elf`: ELF parse and verify path of solana-sbpf, the parser behind on chain program deployment. Seeds are real SPL program binaries in `seeds/sbpf-elf/` plus synthetic minimal ELFs for every sbpf version.
- `block-component`: wincode deserialization of block markers, the wire format carrying block footers, headers and update-parent markers between leader and replay. Seeds are synthetic valid markers for every variant, including footers carrying reward certificates.
- `votor-wire`: the hand written unsafe deserialization of consensus votes and certificates, the first gate for all network input into votor. Seeds are synthetic signed vote messages generated from a fixed keypair, so the corpus is byte-identical across runs.
- `shred-ingress`: shred deserialization, the wire format every peer receives from turbine before blocks exist. Seeds are synthetic merkle shreds carrying valid `ShredVariant` bytes (`MerkleData`/`MerkleCode`, proof size 6) at exact wire sizes: 1203 bytes for data, 1228 for coding. The shred version field is not validated by this parser; turbine filters it downstream.

## Limitations

- The mutator keeps inputs at 64 bytes or larger, so panics that only trigger on shorter inputs are out of reach for this engine.
- The oracle catches Rust panics. Stack exhaustion and aborts from C dependencies are not findings here.
- Footers are seeded with reward certificates; the finalization certificate payload (`VotesAggregate`) is not constructible outside solana-entry's dev feature, so that one nested path is only reached by mutation.

## Adding a target

Implement the `Target` trait: a name, a description, a seed corpus, and a run function. The engine, the mutator, and the oracle are shared. Register it in `targets/mod.rs` and the CLI picks it up.

## Rules of the house

- A panic is a finding. A clean rejection is not.
- Every claim carries a receipt. No receipt, no claim.
- Seeds come from the real world. Synthetic seeds only open the door.
- Everything here runs locally against open source code. Findings in production parsers go to the project security policy first, never straight to the public findings folder.
