#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]

use rand::prelude::*;


/**
 * So, we're going to try to create a cipher?
 * We start with the finite field F_2^8, so we're working with u8's
 * Then, we want something that moves things around in this field
 * And, to make it a clock cipher, we want some group action on V = F_2^8 ^ 2^5, so we're working with 256 bit blocks
 * We want to start out with some standardized hash, e.g. SHA3_512, for our key
 * Actually, lets work with 512 bit blocks.
 * So the idea is that for this round, the key totally determines the output
 * Then, for the second round, we do the first round's operations on the 512 bit key, to get a new key
 * Pretty simple
 */

/**
 * Regarding code performance...
 * Using cargo build is slowest possible
 * Using cargo build --release is much much better
 * Using RUSTFLAGS="-C target-cpu=native" cargo build --release is slightly better than the above
 */
#[macro_use]
extern crate timeit;

// external imports

//internal imports
mod util;
use util::*;

mod l1;
use l1::*;

mod l2;
use l2::*;

mod block;
use block::*;

use sha3::{Digest, Sha3_512};

use std::ops::{Add, Mul};

struct Poly<T: Copy>(Vec<T>);

trait Pow {
    fn power(&self, n: usize) -> Self;
}

impl<K> Pow for K
    where K: Mul<Output=K> + Copy
{
    fn power(&self, n: usize) -> Self {
        let mut out = *self;
        for _ in 1..n {
            out = out * *self;
        }

        out
    }
}

impl<T: Copy> Poly<T> {
    fn evaluate<K>(&self, val: K) -> K
    where K: Pow + Mul<T, Output=K> + Add<Output=K>,
    {
        val
    }
}


struct FFE(u128);

fn main() {

    println!("{}", 2_u128.power(7));


}
