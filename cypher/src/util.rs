use rand::prelude::*;

use sha3::{Digest, Sha3_512};
use std::convert::TryInto;
use std::ops::BitXor;

pub const fac_16: u64 = 20922789888000;

pub type Block16 = [u8; 16];
pub type Key64 = [u8; 64];

pub fn array_bitxor<T: BitXor<Output=T> + Copy, const n: usize>(lhs: &[T;n], rhs: &[T;n]) -> [T;n] {
    let mut out: [T;n] = [lhs[0];n];

    for i in 0..n {
        out[i] = lhs[i] ^ rhs[i];
    }

    out
}

pub fn next_u8(rng: &mut ThreadRng) -> u8 {
    (rng.next_u32() % 256) as u8 // rng.fill_bytes(dest: &mut [u8])
}



pub fn gen_rand_text_slower(size: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(size);

    let mut rng = thread_rng();
    for _ in 0..size {
        out.push(next_u8(&mut rng));
    }

    out
}

pub fn gen_rand_text_med(size: usize) -> Vec<u8> {
    let mut rng = thread_rng();

    (0..size).map(|_| next_u8(&mut rng)).collect()
}

pub fn gen_rand_block(rng: &mut ThreadRng) -> Block16 {
    let mut out: Block16 = [1; 16];

    for i in 0..16 {
        out[i] = next_u8(rng);
    }

    out
}

pub fn gen_rand_key(rng: &mut ThreadRng) -> Key64 {
    let mut out: Key64 = [1; 64];

    for i in 0..64 {
        out[i] = next_u8(rng);
    }

    out
}

pub fn gen_rand_text_blocks(num_blocks: usize) -> Vec<Block16> {
    let mut rng = thread_rng();
    let mut out = Vec::with_capacity(num_blocks);

    for _ in 0..num_blocks {
        out.push(gen_rand_block(&mut rng));
    }

    out
}

pub fn ithPermutation<const n: usize>(mut i: u64) -> [usize; n] {
    //let (mut j, mut k) = (0, 0);
    //int *fact = (int *)calloc(n, sizeof(int));
    let mut fact: [u64; n] = [1; n];
    //int *perm = (int *)calloc(n, sizeof(int));
    let mut perm = [1; n];

    // compute factorial numbers
    /*
    fact[k] = 1;
    while k < n {
        fact[k] = fact[k - 1] * k;
        k += 1;
    }*/

    for idx in 1..n {
        fact[idx] = fact[idx - 1] * (idx as u64);
    }

    // compute factorial code
    for k in 0..n {
        perm[k] = i / fact[n - 1 - k];
        i = i % fact[n - 1 - k];
    }

    // readjust values to obtain the permutation
    // start from the end and check if preceding values are lower
    /*for (k = n - 1; k > 0; --k)
    for (j = k - 1; j >= 0; --j)
        if (perm[j] <= perm[k])
        perm[k]++;*/

    for k in (1..n).rev() {
        for j in (0..k).rev() {
            if perm[j] <= perm[k] {
                perm[k] += 1;
            }
        }
    }

    /*
    k = n - 1;
    while k > 0 {
        j = k - 1;
        while j >= 0 {
            if (perm[j] <= perm[k]) {
                perm[k] += 1;
            }
            j -= 1;
        }
        k -= 1;
    }*/

    //println!("{:?}", &perm);
    //println!("{:?}\n", &fact);
    let mut out = [1_usize; n];
    for i in 0..n {
        out[i] = perm[i] as usize;
    }
    out
}

pub fn invert_perm<const n: usize>(input: &[usize; n]) -> [usize; n] {
    let mut out: [usize; n] = [1; n];

    for idx in 0..n {
        out[input[idx]] = idx;
    }

    out
}

pub fn compose_perm(f: &[usize; 16], g: &[usize; 16]) -> [usize; 16] {
    // computes f(g(x))
    let mut out = [1_usize; 16];

    for i in 0..16 {
        out[i] = f[g[i]];
    }

    out
}

pub fn fac(n: u64) -> u64 {
    if n == 0 {
        1
    } else {
        n * fac(n - 1)
    }
}

pub fn string_to_blocks<T: ToString>(input: T) -> Vec<Block16> {
    let mut out = Vec::new();

    let mut modded_string = input.to_string();

    if modded_string.len() % 16 != 0 {
        for _ in 0..(16 - modded_string.len() % 16) {
            modded_string.push('=');
        }
    }

    // println!("[internal]: {}", &modded_string);
    let l = modded_string.len();
    let bytes = modded_string.into_bytes();
    for idx in 0..l / 16 {
        out.push((&bytes[16 * idx..16 * (idx + 1)]).try_into().unwrap());
    }

    out
}

pub fn blocks_to_string(input: &Vec<Block16>) -> String {
    let mut out = "".to_owned();

    for i in input {
        out.push_str(&String::from_utf8_lossy(i));
    }

    out
}
