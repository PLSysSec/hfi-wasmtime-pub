mod mpk;
use crate::mpk::*;
use libc::{PROT_READ, PROT_WRITE};

// sets the pkru to only allow region 0 + region `pkey`
// disables read and write access to every other region
fn mpk_exclusive_allow(pkey: u32) {
    // bit hacking to enable region 0 + region pkey only
    // let pkru: u32 = !((0x3) << (2 * pkey) | 0x3);
    // wrpkru(pkru);
}

pub fn colorguard_enter_sbox(pkey: u32) {
    // mpk_exclusive_allow(pkey);
}

pub fn colorguard_color_memory(membase: *const u8, slot_size: usize, max_instances: usize) {
    // for idx in 0..max_instances {
    //     let pkey = ((idx % 15) + 1) as u32; // 1-15
    //     let offset = idx.checked_mul(slot_size).unwrap();
    //     // apparently there is not builtin checked addition for pointers in Rust?
    //     colorguard_color_slot(unsafe { membase.add(offset) }, slot_size, pkey);
    // }
}

pub fn colorguard_allocate_pkeys() -> [u32; 16] {
    let mut pkeys: [u32; 16] = [0; 16];
    // for idx in 1..16 {
    //     let pkey = pkey_alloc(0, 0); // create pkeys that do not yet have restrictions
    //     pkeys[idx] = pkey;
    // }
    pkeys
}

pub fn colorguard_deallocate_pkeys(pkeys: [u32; 16]) {
    // free all allocated pkeys
    // for idx in 1..16 {
    //     if pkeys[idx] != 0 {
    //         println!("pkey_free({:?})", pkeys[idx]);
    //         pkey_free(pkeys[idx]);
    //     }
    // }
}

pub fn colorguard_color_slot(linmem_base: *const u8, slot_size: usize, pkey: u32) {
    // let prot = PROT_READ | PROT_WRITE;
    // pkey_mprotect(linmem_base as u64, slot_size, prot, pkey);
}
