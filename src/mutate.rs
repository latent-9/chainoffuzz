use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

pub struct Mutator {
    rng: SmallRng,
}

impl Mutator {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    pub fn mutate(&mut self, data: &[u8]) -> Vec<u8> {
        let mut d = data.to_vec();
        if d.len() < 64 {
            d.extend((0..64 - d.len()).map(|_| self.rng.gen::<u8>()));
        }
        let n = self.rng.gen_range(1..=8);
        for _ in 0..n {
            match self.rng.gen_range(0..8) {
                0 => {
                    let i = self.rng.gen_range(0..d.len());
                    d[i] ^= 1 << self.rng.gen_range(0..8);
                }
                1 => {
                    let i = self.rng.gen_range(0..d.len());
                    d[i] = self.rng.gen();
                }
                2 => {
                    let i = self.rng.gen_range(0..64.min(d.len()));
                    d[i] = self.rng.gen();
                }
                3 => {
                    if d.len() >= 56 {
                        let f: u64 = self.rng.gen();
                        d[48..56].copy_from_slice(&f.to_le_bytes());
                    }
                }
                4 => {
                    if d.len() > 64 {
                        let cut = self.rng.gen_range(64..d.len());
                        d.truncate(cut);
                    }
                }
                5 => {
                    let extra: usize = self.rng.gen_range(1..512);
                    d.extend((0..extra).map(|_| self.rng.gen::<u8>()));
                }
                6 => {
                    if d.len() > 8 {
                        let a = self.rng.gen_range(0..d.len());
                        let b = self.rng.gen_range(0..d.len());
                        d.swap(a, b);
                    }
                }
                _ => {
                    if d.len() > 16 {
                        let src = self.rng.gen_range(0..d.len());
                        let dst = self.rng.gen_range(0..d.len());
                        let max_len = 16.min(d.len() - src);
                        let len = self.rng.gen_range(1..=max_len);
                        let chunk: Vec<u8> = d[src..src + len].to_vec();
                        let dst_end = (dst + len).min(d.len());
                        d[dst..dst_end].copy_from_slice(&chunk[..dst_end - dst]);
                    }
                }
            }
        }
        d
    }

    pub fn pick<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        let i = self.rng.gen_range(0..slice.len());
        &slice[i]
    }
}
