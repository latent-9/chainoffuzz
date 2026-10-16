pub mod sbpf_elf;
pub mod block_component;
pub mod votor_wire;

pub trait Target {
    fn name(&self) -> &'static str;
    fn describe(&self) -> &'static str;
    fn seeds(&self) -> Vec<Vec<u8>>;
    fn run(&self, input: &[u8]) -> Result<(), String>;
}

pub fn all() -> Vec<Box<dyn Target>> {
    vec![
        Box::new(sbpf_elf::SbpfElf),
        Box::new(block_component::BlockComponent),
        Box::new(votor_wire::VotorWire),
    ]
}
