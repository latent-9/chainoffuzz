use super::Target;
use solana_ledger::shred::Shred;
use std::path::Path;

/// Synthetic shred: 64-byte signature placeholder, the shred type tag,
/// slot and index headers, padded to a realistic payload size. The exact
/// merkle layout is explored by the mutator from these anchors.
fn synthetic_shred(tag: u8, slot: u64, index: u32, size: usize) -> Vec<u8> {
    let mut s = vec![0u8; size.max(1200)];
    s[64] = tag;
    s[65..73].copy_from_slice(&slot.to_le_bytes());
    s[73..77].copy_from_slice(&index.to_le_bytes());
    s
}

pub struct ShredIngress;

impl Target for ShredIngress {
    fn name(&self) -> &'static str {
        "shred-ingress"
    }

    fn describe(&self) -> &'static str {
        "shred deserialization, the wire format every peer receives from turbine before blocks exist"
    }

    fn seeds(&self) -> Vec<Vec<u8>> {
        let mut seeds = vec![
            synthetic_shred(0b1010_0101, 100, 0, 1200), // data
            synthetic_shred(0b1010_0101, 1, 0, 1200),
            synthetic_shred(0b1010_0101, u64::MAX, u32::MAX, 1200),
            synthetic_shred(0b0101_1010, 100, 0, 1200), // coding
            synthetic_shred(0b1010_0101, 100, 0, 64),   // truncated
            vec![0xA5u8; 1200],
            vec![0x5Au8; 1200],
            vec![0u8; 1200],
        ];
        let dir = Path::new("seeds/shred-ingress");
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(bytes) = std::fs::read(entry.path()) {
                    seeds.push(bytes);
                }
            }
        }
        seeds
    }

    fn run(&self, input: &[u8]) -> Result<(), String> {
        crate::oracle::guard(move || {
            let _shred = Shred::new_from_serialized_shred(input.to_vec())
                .map_err(|e| format!("deserialize: {e}"))?;
            Ok(())
        })
    }
}
