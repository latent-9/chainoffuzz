use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

/// Runs `f` under a panic guard. A panic becomes Err("PANIC: ..."),
/// everything else passes through unchanged.
pub fn guard<F: FnOnce() -> Result<(), String>>(f: F) -> Result<(), String> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(inner) => inner,
        Err(panic) => {
            let msg = panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown panic".to_string());
            Err(format!("PANIC: {msg}"))
        }
    }
}

/// Archives a caught panic as a finding and returns the file path.
/// The panic message is written next to the input so the finding is
/// self describing.
pub fn archive(dir: &Path, tag: &str, data: &[u8], msg: &str, iter: usize) -> String {
    std::fs::create_dir_all(dir).ok();
    let name = format!("{}/found_{iter}_{tag}", dir.display());
    std::fs::write(&name, data).ok();
    std::fs::write(format!("{name}.txt"), msg).ok();
    name
}
