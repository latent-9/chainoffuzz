use super::Target;
use solana_ledger::shred::Shred;
use std::path::Path;

// Wire anchors for solana-ledger 4.5.0-alpha.2, checked against
// ShredVariant::try_from and Shred{Data,Code}::from_payload:
//   [0..64]   leader signature
//   [64]      shred variant byte: 0b0110_???? = MerkleCode, 0b1001_???? =
//             MerkleData (high nibble); low nibble = merkle proof size.
//             The ShredType tags 0b1010_0101/0b0101_1010 are rejected by
//             ShredVariant::try_from and must not be used here.
//   [65..73]  slot   [73..77] index   [77..79] version   [79..83] fec_set_index
//   data:     [83..85] parent_offset  [85] flags  [86..88] size (an offset
//             from payload start, valid range is headers..=headers+capacity)
//   coding:   [83..85] num_data_shreds  [85..87] num_coding_shreds
//             [87..89] position
// Sizes are exact: ShredData::SIZE_OF_PAYLOAD = 1203, ShredCode = 1228.
const MERKLE_DATA_PROOF_6: u8 = 0b1001_0110;
const MERKLE_CODE_PROOF_6: u8 = 0b0110_0110;
const DATA_SHRED_SIZE: usize = 1203;
const CODING_SHRED_SIZE: usize = 1228;
const DATA_BUFFER_FULL: u16 = 979; // 88 header bytes + 891 capacity at proof 6
const LAST_SHRED_IN_SLOT: u8 = 0b1100_0000; // implies DATA_COMPLETE_SHRED

fn data_shred(slot: u64, index: u32, flags: u8, size: u16) -> Vec<u8> {
    let mut s = vec![0u8; DATA_SHRED_SIZE];
    s[64] = MERKLE_DATA_PROOF_6;
    s[65..73].copy_from_slice(&slot.to_le_bytes());
    s[73..77].copy_from_slice(&index.to_le_bytes());
    // version ([77..79]) is not checked by the parser itself; turbine filters
    // it downstream. fec_set_index ([79..83]) stays 0: index minus
    // fec_set_index must stay inside the 32-shred erasure set.
    s[83..85].copy_from_slice(&1u16.to_le_bytes()); // parent_offset
    s[85] = flags;
    s[86..88].copy_from_slice(&size.to_le_bytes());
    s
}

fn coding_shred(slot: u64, index: u32) -> Vec<u8> {
    let mut s = vec![0u8; CODING_SHRED_SIZE];
    s[64] = MERKLE_CODE_PROOF_6;
    s[65..73].copy_from_slice(&slot.to_le_bytes());
    s[73..77].copy_from_slice(&index.to_le_bytes());
    s[83..85].copy_from_slice(&32u16.to_le_bytes()); // num_data_shreds
    s[85..87].copy_from_slice(&32u16.to_le_bytes()); // num_coding_shreds
    s[87..89].copy_from_slice(&0u16.to_le_bytes()); // position in batch
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
            data_shred(100, 0, 0, DATA_BUFFER_FULL),      // mid-slot, full buffer
            data_shred(100, 31, 0, DATA_BUFFER_FULL),     // last shard of the erasure set
            data_shred(1, 0, 0, 88),                      // empty data buffer
            data_shred(u64::MAX, 0, 0, DATA_BUFFER_FULL), // extreme slot
            data_shred(100, 0, LAST_SHRED_IN_SLOT, DATA_BUFFER_FULL),
            coding_shred(100, 0),
            coding_shred(u64::MAX, 0),
            data_shred(100, 0, 0, DATA_BUFFER_FULL)[..64].to_vec(), // truncated header
            vec![MERKLE_DATA_PROOF_6; DATA_SHRED_SIZE],
            vec![MERKLE_CODE_PROOF_6; CODING_SHRED_SIZE],
            vec![0u8; DATA_SHRED_SIZE],
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
