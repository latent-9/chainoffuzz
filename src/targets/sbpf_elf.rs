use super::Target;
use solana_sbpf::{
    ebpf::MM_STACK_START,
    elf::Executable,
    program::BuiltinProgram,
    verifier::RequisiteVerifier,
    vm::{Config, ContextObject},
};
use std::{path::Path, sync::Arc};

struct NullCtx;

impl ContextObject for NullCtx {
    fn consume(&mut self, _amount: u64) {}
    fn get_remaining(&self) -> u64 {
        u64::MAX
    }
    fn active_mapping_ptr(
        &mut self,
    ) -> std::ptr::NonNull<solana_sbpf::memory_region::MemoryMapping> {
        unreachable!("not executed")
    }
}

fn minimal_elf(flags: u64) -> Vec<u8> {
    let mut e = Vec::new();
    e.extend_from_slice(&[0x7f, b'E', b'L', b'F']);
    e.push(2);
    e.push(1);
    e.push(1);
    e.push(0);
    e.extend_from_slice(&[0; 8]);
    e.extend_from_slice(&2u16.to_le_bytes());
    e.extend_from_slice(&247u16.to_le_bytes());
    e.extend_from_slice(&1u32.to_le_bytes());
    e.extend_from_slice(&MM_STACK_START.to_le_bytes());
    e.extend_from_slice(&64u64.to_le_bytes());
    e.extend_from_slice(&0u64.to_le_bytes());
    e.extend_from_slice(&flags.to_le_bytes());
    e.extend_from_slice(&64u16.to_le_bytes());
    e.extend_from_slice(&56u16.to_le_bytes());
    e.extend_from_slice(&1u16.to_le_bytes());
    e.extend_from_slice(&64u16.to_le_bytes());
    e.extend_from_slice(&0u16.to_le_bytes());
    e.extend_from_slice(&0u16.to_le_bytes());
    e.extend_from_slice(&1u32.to_le_bytes());
    e.extend_from_slice(&7u32.to_le_bytes());
    e.extend_from_slice(&0u64.to_le_bytes());
    e.extend_from_slice(&MM_STACK_START.to_le_bytes());
    e.extend_from_slice(&MM_STACK_START.to_le_bytes());
    e.extend_from_slice(&32u64.to_le_bytes());
    e.extend_from_slice(&32u64.to_le_bytes());
    e.extend_from_slice(&4096u64.to_le_bytes());
    e.extend_from_slice(&[0x95u8; 32]);
    e
}

pub struct SbpfElf;

impl Target for SbpfElf {
    fn name(&self) -> &'static str {
        "sbpf-elf"
    }

    fn describe(&self) -> &'static str {
        "ELF parse and verify path of solana-sbpf, the parser behind on chain program deployment"
    }

    fn seeds(&self) -> Vec<Vec<u8>> {
        let mut seeds = vec![
            minimal_elf(0),
            minimal_elf(1),
            minimal_elf(2),
            minimal_elf(3),
            minimal_elf(4),
            minimal_elf(0)[..16].to_vec(),
            (0..512).map(|_| 0).collect(),
            (0..512).map(|_| 0xff).collect(),
        ];
        let dir = Path::new("seeds/sbpf-elf");
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
        let loader = Arc::new(BuiltinProgram::new_loader(Config {
            max_call_depth: 64,
            stack_frame_size: 8192,
            enable_stack_frame_gaps: true,
            ..Config::default()
        }));
        crate::oracle::guard(move || {
            let executable = Executable::<NullCtx>::from_elf(input, loader.clone())
                .map_err(|e| format!("parse: {e}"))?;
            executable
                .verify::<RequisiteVerifier>()
                .map_err(|e| format!("verify: {e}"))?;
            Ok(())
        })
    }
}
