use super::Target;
use agave_votor_messages::reward_certificate::{
    NotarRewardCertificate, SkipRewardCertificate,
};
use solana_entry::block_component::{
    BlockFooterV1, BlockHeaderV1, BlockMarkerV1, UpdateParentV1, VersionedBlockFooter,
    VersionedBlockHeader, VersionedBlockMarker, VersionedUpdateParent,
};
use solana_bls_signatures::SignatureCompressed as BLSSignatureCompressed;
use solana_hash::Hash;
use std::path::Path;

fn seed_footer() -> Vec<u8> {
    let footer = BlockFooterV1 {
        bank_hash: Hash::default(),
        block_producer_time_nanos: 1_700_000_000_000_000_000,
        block_user_agent: b"agave/4.5.0".to_vec(),
        block_final_cert: None,
        skip_reward_cert: None,
        notar_reward_cert: None,
    };
    let marker = BlockMarkerV1::new_block_footer(VersionedBlockFooter::V1(footer));
    wincode::serialize(&VersionedBlockMarker::new(marker)).unwrap()
}

fn seed_footer_with_reward_certs() -> Vec<u8> {
    // block_final_cert stays None: VotesAggregate has private fields and is
    // only constructible behind solana-entry's dev feature. The reward certs
    // still exercise Some(..) certificate payloads inside a footer.
    let signature = BLSSignatureCompressed([7u8; 96]);
    let footer = BlockFooterV1 {
        bank_hash: Hash::default(),
        block_producer_time_nanos: 1_700_000_000_000_000_000,
        block_user_agent: b"agave/4.5.0".to_vec(),
        block_final_cert: None,
        skip_reward_cert: Some(
            SkipRewardCertificate::try_new(100, signature, vec![0xff; 64]).unwrap(),
        ),
        notar_reward_cert: Some(
            NotarRewardCertificate::try_new(100, Hash::default(), signature, vec![0xff; 64])
                .unwrap(),
        ),
    };
    let marker = BlockMarkerV1::new_block_footer(VersionedBlockFooter::V1(footer));
    wincode::serialize(&VersionedBlockMarker::new(marker)).unwrap()
}

fn seed_header() -> Vec<u8> {
    let marker = BlockMarkerV1::new_block_header(VersionedBlockHeader::V1(BlockHeaderV1 {
        parent_slot: 100,
        parent_block_id: Hash::default(),
    }));
    wincode::serialize(&VersionedBlockMarker::new(marker)).unwrap()
}

fn seed_update_parent() -> Vec<u8> {
    let marker = BlockMarkerV1::new_update_parent(VersionedUpdateParent::V1(UpdateParentV1 {
        new_parent_slot: 100,
        new_parent_block_id: Hash::default().into(),
    }));
    wincode::serialize(&VersionedBlockMarker::new(marker)).unwrap()
}

pub struct BlockComponent;

impl Target for BlockComponent {
    fn name(&self) -> &'static str {
        "block-component"
    }

    fn describe(&self) -> &'static str {
        "wincode deserialization of block markers, the wire format for block footers, headers and update-parent markers"
    }

    fn seeds(&self) -> Vec<Vec<u8>> {
        let mut seeds = vec![
            seed_footer(),
            seed_footer_with_reward_certs(),
            seed_header(),
            seed_update_parent(),
        ];
        let dir = Path::new("seeds/block-component");
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
            let _: VersionedBlockMarker =
                wincode::deserialize(input).map_err(|e| format!("deserialize: {e}"))?;
            Ok(())
        })
    }
}
