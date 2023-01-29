// use raw_syscall::{pkey_alloc, pkey_free, pkey_mprotect};
use syscalls::syscall;
use syscalls::Sysno;

pub const PKEY_DISABLE_ACCESS: u64 = 1;

// raw interface should be unsafe
#[inline]
pub fn pkey_alloc(flags: u64, ini_access_rights: u64) -> u32 {
    // let result = unsafe { syscall!(Sysno::pkey_alloc, flags, ini_access_rights) };
    // //assert!(result > 0);
    // result.unwrap() as u32
    1 as u32
}

#[inline]
pub fn pkey_free(pkey: u32) -> i32 {
    // let result = unsafe { syscall!(Sysno::pkey_free, pkey) };
    // //assert!(result == 0);
    // result.unwrap() as i32
    0 as i32
}

#[inline]
pub fn pkey_mprotect(start: u64, len: usize, protection: i32, pkey: u32) -> i32 {
    // let result = unsafe { syscall!(Sysno::pkey_mprotect, start, len, protection, pkey) };
    // //assert!(result == 0);
    // result.unwrap() as i32
    0 as i32
}

#[inline]
pub fn pkey_set(pkey: u32, rights: u64) {
    // let pkru: u32 = (rights as u32) << (2 * pkey);
    // wrpkru(pkru)
}

// https://shell-storm.org/x86doc/RDPKRU.html
#[inline]
pub fn rdpkru() -> u32 {
    // let ecx: u32 = 0;
    // let pkru: u32;
    // unsafe {
    //     std::arch::asm!(
    //         ".byte 0x0f,0x01,0xee\n\t",
    //         in("ecx") ecx,   // ecx must be 0 when rdpkru executes else a GP exception occurs
    //         out("edx") _ ,   // rdpkru clobbers edx
    //         out("eax") pkru, // get the output
    //     );
    // }
    // return pkru;
    0
}

// https://www.felixcloutier.com/x86/wrpkru
#[inline]
pub fn wrpkru(pkru: u32) {
    // let ecx: u32 = 0;
    // let edx: u32 = 0;
    // unsafe {
    //     std::arch::asm!(
    //         ".byte 0x0f,0x01,0xef\n\t",
    //         in("eax") pkru, // eax contains pkey to write
    //         in("ecx") ecx,  // ecx must be 0 else GP exception occurs
    //         in("edx") edx,  // edx must be 0 else GP exception occurs
    //     );
    // }
}

// static inline void __wrpkru(unsigned int pkru)
// {
// 	unsigned int eax = pkru;
// 	unsigned int ecx = 0;
// 	unsigned int edx = 0;

// 	dprintf4("%s() changing %08x to %08x\n", __func__, __rdpkru(), pkru);
// 	asm volatile(".byte 0x0f,0x01,0xef\n\t"
// 		     : : "a" (eax), "c" (ecx), "d" (edx));
// 	assert(pkru == __rdpkru());
// }
