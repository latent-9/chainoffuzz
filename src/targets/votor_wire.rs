use super::Target;
use agave_votor_messages::consensus_message::VoteMessage;
use agave_votor_messages::vote::Vote;
use agave_votor_messages::wire::{
    get_vote_payload_to_sign, VersionedWireConsensusMessage,
};
use solana_bls_signatures::Keypair;
use std::num::NonZero;
use std::path::Path;

fn seed_vote_message(slot: u64) -> Vec<u8> {
    let keypair = Keypair::new();
    let vote = Vote::new_skip_vote(slot);
    let payload = get_vote_payload_to_sign(vote, 7);
    let signature = keypair.sign(&payload).into();
    let msg = VoteMessage {
        vote,
        signature,
        rank: 0,
        stake: NonZero::new(42).unwrap(),
    };
    wincode::serialize(&VersionedWireConsensusMessage::new_from_vote(msg, 7)).unwrap()
}

pub struct VotorWire;

impl Target for VotorWire {
    fn name(&self) -> &'static str {
        "votor-wire"
    }

    fn describe(&self) -> &'static str {
        "the hand written unsafe wire deserialization of consensus votes and certificates, the first gate for all network input into votor"
    }

    fn seeds(&self) -> Vec<Vec<u8>> {
        let mut seeds = vec![
            seed_vote_message(100),
            seed_vote_message(0),
            seed_vote_message(u64::MAX),
        ];
        let dir = Path::new("seeds/votor-wire");
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
            let _: VersionedWireConsensusMessage =
                VersionedWireConsensusMessage::deserialize_with_expected_shred_version(
                    input,
                    wincode::config::Configuration::default(),
                    7,
                )
                .map_err(|e| format!("deserialize: {e}"))?;
            Ok(())
        })
    }
}
