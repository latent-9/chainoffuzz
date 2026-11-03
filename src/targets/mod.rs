pub mod sbpf_elf;
pub mod block_component;
pub mod votor_wire;
pub mod shred_ingress;

pub trait Target {
    fn name(&self) -> &'static str;
    fn describe(&self) -> &'static str;
    fn seeds(&self) -> Vec<Vec<u8>>;
    fn run(&self, input: &[u8]) -> Result<(), String>;
}

pub fn all() -> Vec<Box<dyn Target>> {
    vec![
        Box::new(sbpf_elf::SbpfElf::new()),
        Box::new(block_component::BlockComponent),
        Box::new(votor_wire::VotorWire),
        Box::new(shred_ingress::ShredIngress),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_never_panic() {
        for t in all() {
            for (i, seed) in t.seeds().iter().enumerate() {
                if let Err(msg) = t.run(seed) {
                    assert!(
                        !msg.starts_with("PANIC"),
                        "{} seed #{} panics: {msg}",
                        t.name(),
                        i
                    );
                }
            }
        }
    }

    #[test]
    fn every_target_accepts_at_least_one_seed() {
        for t in all() {
            let seeds = t.seeds();
            assert!(
                seeds.iter().any(|s| t.run(s).is_ok()),
                "{} rejects every one of its own seeds",
                t.name()
            );
        }
    }
}
