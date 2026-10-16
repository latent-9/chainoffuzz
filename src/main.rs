mod mutate;
mod oracle;
mod targets;

use mutate::Mutator;
use std::time::Instant;
use std::{path::Path, process::exit};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let targets = targets::all();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        eprintln!("ChainOfFuzz: mutation fuzzer for consensus adjacent parsers");
        eprintln!();
        eprintln!("usage: chainoffuzz <target> [iterations] [seed]");
        eprintln!();
        eprintln!("targets:");
        for t in &targets {
            eprintln!("  {:<12} {}", t.name(), t.describe());
        }
        exit(if args.len() < 2 { 1 } else { 0 });
    }

    let target = targets
        .iter()
        .find(|t| t.name() == args[1])
        .unwrap_or_else(|| {
            eprintln!("unknown target: {}", args[1]);
            exit(1);
        });

    let iters: usize = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);
    let seed: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0xC0FFEE);

    let findings_dir = Path::new("findings");
    let seeds = target.seeds();
    if seeds.is_empty() {
        eprintln!("no seeds for target {}", target.name());
        exit(1);
    }

    eprintln!(
        "target {} | {} seeds | {} iterations | seed {}",
        target.name(),
        seeds.len(),
        iters,
        seed
    );
    let mut mutator = Mutator::new(seed);
    let mut current = seeds[0].clone();
    let mut accepted = 0usize;
    let mut rejected = 0usize;
    let mut panics = 0usize;
    let start = Instant::now();

    for i in 0..iters {
        if i % 97 == 0 {
            current = mutator.pick(&seeds).clone();
        }
        let data = mutator.mutate(&current);
        match target.run(&data) {
            Err(msg) if msg.starts_with("PANIC") => {
                let tag = format!("p{panics}");
                let path = oracle::archive(findings_dir, &tag, &data, &msg, i);
                eprintln!("=== FIND [{tag}] iter {i}: {msg} -> {path}");
                panics += 1;
                current = data;
            }
            Err(_) => {
                rejected += 1;
                current = data;
            }
            Ok(()) => {
                accepted += 1;
                current = data;
            }
        }
        if i % 1_000_000 == 0 && i > 0 {
            let elapsed = start.elapsed().as_secs_f64();
            eprintln!(
                "iter {i} | accepted {accepted} | rejected {rejected} | PANICS {panics} | {:.0} it/s",
                i as f64 / elapsed
            );
        }
    }

    eprintln!(
        "DONE iters {iters} | accepted {accepted} | rejected {rejected} | PANICS {panics} | {:.1}s",
        start.elapsed().as_secs_f64()
    );
}
