#![allow(dead_code)]

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Clone, Debug)]
pub struct LargePayload {
    pub data: [u64; 64],
}

impl LargePayload {
    pub fn new(seed: u64) -> Self {
        let mut data = [0u64; 64];
        for (index, slot) in data.iter_mut().enumerate() {
            *slot = seed
                .wrapping_mul(1_146_295_123)
                .wrapping_add(index as u64 * 97);
        }
        Self { data }
    }
}

pub fn short_strings(size: usize, seed: u64) -> Vec<String> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..size)
        .map(|index| {
            let suffix = rng.r#gen::<u32>();
            format!("k-{index:05}-{suffix:08x}")
        })
        .collect()
}