use crate::util::*;
use crate::l1::*;
use std::convert::TryInto;

/**
 * So, here the idea is that we have this cipher and
 * we keep it around for whenever we need to encrypt or decrypt something.
 * It's basically a wrapper over the key, plus a bunch of functions
 */
pub struct L2Cypher {
    key: Key64,
}

impl L2Cypher {
    pub fn new(key: Key64) -> Self {
        Self {
            key: key,
        }
    }

    pub fn encrypt_single_block(&self, input: &Block16) -> Block16 {
        let a1: Block16 = self.key[0..16].try_into().unwrap();
        let a2: Block16 = self.key[16..32].try_into().unwrap();
        let a3: Block16 = self.key[32..48].try_into().unwrap();
        let a4: Block16 = self.key[48..64].try_into().unwrap();

        let mut p1n: u64 = 1;
        let mut p2n: u64 = 1;
        let mut p3n: u64 = 1;
        let mut p4n: u64 = 1;

        for i in 0..64 {
            let tmp = i % 4;
            if tmp == 0 {
                p1n = p1n.wrapping_mul(self.key[i] as u64);
            } else if tmp == 1 {
                p2n = p2n.wrapping_mul(self.key[i] as u64);
            } else if tmp == 2 {
                p3n = p3n.wrapping_mul(self.key[i] as u64);
            } else {
                p4n = p4n.wrapping_mul(self.key[i] as u64);
            }
        }

        let perm1: [usize; 16] = ithPermutation::<16>(p1n % fac_16);
        let perm2: [usize; 16] = ithPermutation::<16>(p2n % fac_16);
        let perm3: [usize; 16] = ithPermutation::<16>(p3n % fac_16);
        let perm4: [usize; 16] = ithPermutation::<16>(p4n % fac_16);

        let mut tmp = round_subroutine(input, &a1, &perm1);
        tmp = round_subroutine(&tmp, &a2, &perm2);
        tmp = round_subroutine(&tmp, &a3, &perm3);
        round_subroutine(&tmp, &a4, &perm4)
    }

    pub fn decrypt_single_block(&self, input: &Block16) -> Block16 {
        let a1: Block16 = self.key[0..16].try_into().unwrap();
        let a2: Block16 = self.key[16..32].try_into().unwrap();
        let a3: Block16 = self.key[32..48].try_into().unwrap();
        let a4: Block16 = self.key[48..64].try_into().unwrap();

        let mut p1n: u64 = 1;
        let mut p2n: u64 = 1;
        let mut p3n: u64 = 1;
        let mut p4n: u64 = 1;

        for i in 0..64 {
            let tmp = i % 4;
            if tmp == 0 {
                p1n = p1n.wrapping_mul(self.key[i] as u64);
            } else if tmp == 1 {
                p2n = p2n.wrapping_mul(self.key[i] as u64);
            } else if tmp == 2 {
                p3n = p3n.wrapping_mul(self.key[i] as u64);
            } else {
                p4n = p4n.wrapping_mul(self.key[i] as u64);
            }
        }

        let perm1: [usize; 16] = ithPermutation::<16>(p1n % fac_16);
        let perm2: [usize; 16] = ithPermutation::<16>(p2n % fac_16);
        let perm3: [usize; 16] = ithPermutation::<16>(p3n % fac_16);
        let perm4: [usize; 16] = ithPermutation::<16>(p4n % fac_16);

        let perm1inv: [usize; 16] = invert_perm::<16>(&perm1);
        let perm2inv: [usize; 16] = invert_perm::<16>(&perm2);
        let perm3inv: [usize; 16] = invert_perm::<16>(&perm3);
        let perm4inv: [usize; 16] = invert_perm::<16>(&perm4);

        let mut tmp = inv_round_subroutine(input, &a4, &perm4inv);
        tmp = inv_round_subroutine(&tmp, &a3, &perm3inv);
        tmp = inv_round_subroutine(&tmp, &a2, &perm2inv);
        inv_round_subroutine(&tmp, &a1, &perm1inv)
    }
}