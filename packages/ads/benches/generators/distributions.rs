#![allow(dead_code)]

use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Clone, Copy)]
pub enum MapOp {
    Contains(u64),
    Upsert(u64, u64),
    Remove(u64),
}

pub fn uniform_keys(size: usize, seed: u64) -> Vec<u64> {
    let mut keys: Vec<u64> = (0..size as u64).collect();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    keys.shuffle(&mut rng);
    keys
}

pub fn sorted_keys(size: usize) -> Vec<u64> {
    (0..size as u64).collect()
}

pub fn zipfian_queries(size: usize, count: usize, seed: u64) -> Vec<u64> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let hot_cap = usize::max(1, size / 5) as u64;
    let full = size as u64;
    (0..count)
        .map(|_| {
            if rng.gen_bool(0.8) {
                rng.gen_range(0..hot_cap)
            } else {
                rng.gen_range(0..full)
            }
        })
        .collect()
}

pub fn temporal_locality_queries(size: usize, count: usize, seed: u64) -> Vec<u64> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut window: Vec<u64> = (0..usize::min(32, size)).map(|x| x as u64).collect();
    let mut next_fresh = window.len() as u64;

    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        if rng.gen_bool(0.85) && !window.is_empty() {
            let idx = rng.gen_range(0..window.len());
            result.push(window[idx]);
        } else {
            let key = if next_fresh < size as u64 {
                let fresh = next_fresh;
                next_fresh += 1;
                fresh
            } else {
                rng.gen_range(0..size as u64)
            };

            if window.len() >= 64 {
                let drop_idx = rng.gen_range(0..window.len());
                window.swap_remove(drop_idx);
            }

            window.push(key);
            result.push(key);
        }
    }

    result
}

pub fn read_heavy_ops(size: usize, count: usize, seed: u64) -> Vec<MapOp> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let queries = zipfian_queries(size, count, seed ^ 0xA5A5_A5A5_A5A5_A5A5);
    queries
        .into_iter()
        .map(|key| {
            if rng.gen_bool(0.95) {
                MapOp::Contains(key)
            } else {
                MapOp::Upsert(key, key ^ 0xDEAD_BEEF)
            }
        })
        .collect()
}

pub fn write_heavy_ops(size: usize, count: usize, seed: u64) -> Vec<MapOp> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let max_key = (size as u64).saturating_mul(2);

    (0..count)
        .map(|_| {
            let key = rng.gen_range(0..max_key);
            let selector = rng.gen_range(0u8..100u8);
            if selector < 45 {
                MapOp::Upsert(key, key.wrapping_mul(31))
            } else if selector < 90 {
                MapOp::Remove(key)
            } else {
                MapOp::Contains(key)
            }
        })
        .collect()
}
