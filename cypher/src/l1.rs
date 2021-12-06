/**
 * So, this cipher is actually pretty bad
 * It's vulnerable to the same thing that Computerphile used to break enigma
 * Using a key that's very close but not exactly the same as the right key
 * yields plaintext that is very close but not exactly the same as the right plaintext
 */
use crate::util::*;
use rayon::prelude::*;
use std::convert::TryInto;

pub fn round_add(target: &[u8; 16], add_round_key: &[u8; 16]) -> [u8; 16] {
    let mut out: [u8; 16] = [0; 16];

    for i in 0..16 {
        out[i] = target[i].wrapping_add(add_round_key[i]);
    }

    out
}

pub fn round_sub(target: &[u8; 16], add_round_key: &[u8; 16]) -> [u8; 16] {
    let mut out: [u8; 16] = [0; 16];

    for i in 0..16 {
        out[i] = target[i].wrapping_add((u8::MAX - add_round_key[i]).wrapping_add(1));
    }

    out
}

// you need 44.25014046988262 bits to store 16 permutations
pub fn round_perm(target: &[u8; 16], perm_round: &[usize; 16]) -> [u8; 16] {
    // let p = ithPermutation::<16>(*perm_round_key); // here we assume that perm_round_key is in fact legal, that it is less than 16!
    let mut out: [u8; 16] = [0; 16];
    for idx in 0..16 {
        out[idx] = target[perm_round[idx]];
    }

    out
}

pub fn enc_text<T: ToString>(input: T, key: &[u8; 64]) -> String {
    let blocks = string_to_blocks(input);

    let enc_blocks: Vec<Block16> = blocks.iter().map(|x| three_perm_enc(x, key)).collect();
    blocks_to_string(&enc_blocks)
}

pub fn dec_text<T: ToString>(input: T, key: &[u8; 64]) -> String {
    let blocks = string_to_blocks(input);

    let dec_blocks: Vec<Block16> = blocks.iter().map(|x| three_perm_dec(x, key)).collect();
    blocks_to_string(&dec_blocks)
}

pub fn enc_blocks(input: &Vec<Block16>, key: &[u8; 64]) -> Vec<Block16> {
    input.iter().map(|x| three_perm_enc(x, key)).collect()
}

pub fn dec_blocks(input: &Vec<Block16>, key: &[u8; 64]) -> Vec<Block16> {
    input.iter().map(|x| three_perm_dec(x, key)).collect()
}

pub fn p_enc_blocks(input: &Vec<Block16>, key: &[u8; 64]) -> Vec<Block16> {
    input.par_iter().map(|x| three_perm_enc(x, key)).collect()
}

pub fn p_dec_blocks(input: &Vec<Block16>, key: &[u8; 64]) -> Vec<Block16> {
    input.par_iter().map(|x| three_perm_dec(x, key)).collect()
}

pub fn round_subroutine(target: &Block16, add_round_key: &Block16, perm: &[usize; 16]) -> [u8; 16] {
    let tmp = round_add(target, add_round_key);
    round_perm(&tmp, perm)
}

pub fn inv_round_subroutine(
    target: &Block16,
    add_round_key: &Block16,
    invperm: &[usize; 16],
) -> Block16 {
    let tmp = round_perm(target, invperm);
    round_sub(&tmp, add_round_key)
}

pub fn three_perm_enc(target: &Block16, key: &[u8; 64]) -> Block16 {
    let a1: Block16 = key[0..16].try_into().unwrap();
    let a2: Block16 = key[16..32].try_into().unwrap();
    let a3: Block16 = key[32..48].try_into().unwrap();

    let pnum1_bytes: [u8; 8] = key[48..56].try_into().unwrap();
    let pnum2_bytes: [u8; 8] = key[56..64].try_into().unwrap();

    let pnum1: u64 = u64::from_le_bytes(pnum1_bytes) % fac_16;
    let pnum2: u64 = u64::from_le_bytes(pnum2_bytes) % fac_16;

    let perm1: [usize; 16] = ithPermutation::<16>(pnum1);
    let perm2: [usize; 16] = ithPermutation::<16>(pnum2);
    let perm3: [usize; 16] = compose_perm(&perm1, &perm2);
    /*
     * Clunky and doesn't really increase the protection of the cipher, but better than just ending on an add, I think
     * At the very least, the addition doesn't commute with the G action on V
     */

    let mut tmp = round_subroutine(target, &a1, &perm1);
    tmp = round_subroutine(&tmp, &a2, &perm2);
    round_subroutine(&tmp, &a3, &perm3)
}

pub fn three_perm_dec(target: &Block16, key: &[u8; 64]) -> Block16 {
    let a1: Block16 = key[0..16].try_into().unwrap();
    let a2: Block16 = key[16..32].try_into().unwrap();
    let a3: Block16 = key[32..48].try_into().unwrap();

    let pnum1_bytes: [u8; 8] = key[48..56].try_into().unwrap();
    let pnum2_bytes: [u8; 8] = key[56..64].try_into().unwrap();

    let fac16 = fac_16;
    let pnum1: u64 = u64::from_le_bytes(pnum1_bytes) % fac16;
    let pnum2: u64 = u64::from_le_bytes(pnum2_bytes) % fac16;

    let perm1: [usize; 16] = ithPermutation::<16>(pnum1);
    let perm2: [usize; 16] = ithPermutation::<16>(pnum2);
    let perm3: [usize; 16] = compose_perm(&perm1, &perm2);

    let inv_perm1 = invert_perm(&perm1);
    let inv_perm2 = invert_perm(&perm2);
    let inv_perm3 = invert_perm(&perm3);

    /*
     * Clunky and doesn't really increase the protection of the cipher, but better than just ending on an add, I think
     * At the very least, the addition doesn't commute with the G action on V
     */

    let mut tmp = inv_round_subroutine(target, &a3, &inv_perm3);
    tmp = inv_round_subroutine(&tmp, &a2, &inv_perm2);
    inv_round_subroutine(&tmp, &a1, &inv_perm1)
}
