use crate::util::{ithPermutation, Block16, Key64, fac_16, compose_perm, invert_perm};
use crate::l1::round_perm;
use std::convert::TryInto;

/**
 * So, this is a new cipher. We want everything to be in GL(V), so we can rely upon eigenvectors to assert that there are very few fixed points
 */

fn forward_subround(block: &Block16, perm: &[usize;16], raw_mult: &Block16) -> Block16 {
    let mut tmp = round_perm(block, perm);
    for idx in 0..16 {
        tmp[idx] = tmp[idx].wrapping_mul((raw_mult[idx] % (u8::MAX-1)) + 1);
    }

    tmp
}

fn backwards_subround(block: &Block16, invperm: &[usize;16], forward_raw_mult: &Block16) -> Block16 {
    let mut tmp = [1_u8;16];

    for idx in 0..16 {
        tmp[idx] = block[idx].wrapping_mul((forward_raw_mult[idx] % (u8::MAX-1)) + 1);
    }
    tmp = round_perm(&tmp, invperm);

    tmp
}

fn forward_round(round_key: &Key64, input: &Block16) -> Block16 {
    // we permute, multiply, permute, multiply, etc etc etc

    /*
     * >perm
     * >mult
     * >perm
     * >mult
     * >perm
     * >mult
     * >perm
     * >mult
     */

    let a1: Block16 = round_key[0..16].try_into().unwrap();
    let a2: Block16 = round_key[16..32].try_into().unwrap();
    let a3: Block16 = round_key[32..48].try_into().unwrap();
    let a4: Block16 = round_key[48..].try_into().unwrap();

    let mut v = round_key.iter();

    let pnum1_bytes: [u8; 8] = v.clone().step_by(4).map(|x| *x).collect::<Vec<u8>>().try_into().unwrap();
    v.next();
    let pnum2_bytes: [u8; 8] = v.clone().step_by(4).map(|x| *x).collect::<Vec<u8>>().try_into().unwrap();
    v.next();
    let pnum3_bytes: [u8; 8] = v.clone().step_by(4).map(|x| *x).collect::<Vec<u8>>().try_into().unwrap();
    v.next();
    let pnum4_bytes: [u8; 8] = v.clone().step_by(4).map(|x| *x).collect::<Vec<u8>>().try_into().unwrap();

    let pnum1: u64 = u64::from_le_bytes(pnum1_bytes) % fac_16;
    let pnum2: u64 = u64::from_le_bytes(pnum2_bytes) % fac_16;
    let pnum3: u64 = u64::from_le_bytes(pnum3_bytes) % fac_16;
    let pnum4: u64 = u64::from_le_bytes(pnum4_bytes) % fac_16;

    let perm1: [usize; 16] = ithPermutation::<16>(pnum1);
    let perm2: [usize; 16] = ithPermutation::<16>(pnum2);
    let perm3: [usize; 16] = ithPermutation::<16>(pnum3);
    let perm4: [usize; 16] = ithPermutation::<16>(pnum4);

    let mut tmp = forward_subround(input, &perm1, &a1);
    tmp = forward_subround(&tmp, &perm2, &a2);
    tmp = forward_subround(&tmp, &perm3, &a3);
    tmp = forward_subround(&tmp, &perm4, &a4);

    tmp
}

fn backward_round(round_key: &Key64, input: &Block16) -> Block16 {
    // we permute, multiply, permute, multiply, etc etc etc, but in reverse

    /*
     * >perm
     * >mult
     * >perm
     * >mult
     * >perm
     * >mult
     * >perm
     * >mult
     * but in reverse
     */

    let a1: Block16 = round_key[0..16].try_into().unwrap();
    let a2: Block16 = round_key[16..32].try_into().unwrap();
    let a3: Block16 = round_key[32..48].try_into().unwrap();
    let a4: Block16 = round_key[48..].try_into().unwrap();

    let mut v = round_key.iter();

    let pnum1_bytes: [u8; 8] = v.clone().step_by(4).map(|x| *x).collect::<Vec<u8>>().try_into().unwrap();
    v.next();
    let pnum2_bytes: [u8; 8] = v.clone().step_by(4).map(|x| *x).collect::<Vec<u8>>().try_into().unwrap();
    v.next();
    let pnum3_bytes: [u8; 8] = v.clone().step_by(4).map(|x| *x).collect::<Vec<u8>>().try_into().unwrap();
    v.next();
    let pnum4_bytes: [u8; 8] = v.clone().step_by(4).map(|x| *x).collect::<Vec<u8>>().try_into().unwrap();

    let pnum1: u64 = u64::from_le_bytes(pnum1_bytes) % fac_16;
    let pnum2: u64 = u64::from_le_bytes(pnum2_bytes) % fac_16;
    let pnum3: u64 = u64::from_le_bytes(pnum3_bytes) % fac_16;
    let pnum4: u64 = u64::from_le_bytes(pnum4_bytes) % fac_16;

    let perm1: [usize; 16] = ithPermutation::<16>(pnum1);
    let perm2: [usize; 16] = ithPermutation::<16>(pnum2);
    let perm3: [usize; 16] = ithPermutation::<16>(pnum3);
    let perm4: [usize; 16] = ithPermutation::<16>(pnum4);

    let inv_perm_1: [usize; 16] = invert_perm(&perm1);
    let inv_perm_2: [usize; 16] = invert_perm(&perm2);
    let inv_perm_3: [usize; 16] = invert_perm(&perm3);
    let inv_perm_4: [usize; 16] = invert_perm(&perm4);

    let mut tmp = forward_subround(input, &perm1, &a1);
    tmp = forward_subround(&tmp, &perm2, &a2);
    tmp = forward_subround(&tmp, &perm3, &a3);
    tmp = forward_subround(&tmp, &perm4, &a4);

    tmp

}
