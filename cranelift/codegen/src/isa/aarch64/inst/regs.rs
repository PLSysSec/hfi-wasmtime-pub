//! AArch64 ISA definitions: registers.

use crate::isa::aarch64::inst::OperandSize;
use crate::isa::aarch64::inst::ScalarSize;
use crate::isa::aarch64::inst::VectorSize;
use crate::machinst::AllocationConsumer;
use crate::machinst::RealReg;
use crate::machinst::{Reg, RegClass, Writable};
use crate::settings;
use regalloc2::MachineEnv;
use regalloc2::PReg;
use regalloc2::VReg;

use std::string::{String, ToString};

//=============================================================================
// Registers, the Universe thereof, and printing

/// The pinned register on this architecture.
/// It must be the same as Spidermonkey's HeapReg, as found in this file.
/// https://searchfox.org/mozilla-central/source/js/src/jit/arm64/Assembler-arm64.h#103
pub const PINNED_REG: u8 = 21;

/// Get a reference to an X-register (integer register). Do not use
/// this for xsp / xzr; we have two special registers for those.
pub fn xreg(num: u8) -> Reg {
    Reg::from(xreg_preg(num))
}

/// Get the given X-register as a PReg.
pub(crate) const fn xreg_preg(num: u8) -> PReg {
    assert!(num < 31);
    PReg::new(num as usize, RegClass::Int)
}

/// Get a writable reference to an X-register.
pub fn writable_xreg(num: u8) -> Writable<Reg> {
    Writable::from_reg(xreg(num))
}

/// Get a reference to a V-register (vector/FP register).
pub fn vreg(num: u8) -> Reg {
    Reg::from(vreg_preg(num))
}

/// Get the given V-register as a PReg.
pub(crate) const fn vreg_preg(num: u8) -> PReg {
    assert!(num < 32);
    PReg::new(num as usize, RegClass::Float)
}

/// Get a writable reference to a V-register.
#[cfg(test)] // Used only in test code.
pub fn writable_vreg(num: u8) -> Writable<Reg> {
    Writable::from_reg(vreg(num))
}

/// Get a reference to the zero-register.
pub fn zero_reg() -> Reg {
    let preg = PReg::new(31, RegClass::Int);
    Reg::from(VReg::new(preg.index(), RegClass::Int))
}

/// Get a writable reference to the zero-register (this discards a result).
pub fn writable_zero_reg() -> Writable<Reg> {
    Writable::from_reg(zero_reg())
}

/// Get a reference to the stack-pointer register.
pub fn stack_reg() -> Reg {
    // XSP (stack) and XZR (zero) are logically different registers
    // which have the same hardware encoding, and whose meaning, in
    // real aarch64 instructions, is context-dependent. For extra
    // correctness assurances and for correct printing, we make them
    // be two different real registers from a regalloc perspective.
    //
    // We represent XZR as if it were xreg(31); XSP is xreg(31 +
    // 32). The PReg bit-packing allows 6 bits (64 registers) so we
    // make use of this extra space to distinguish xzr and xsp. We
    // mask off the 6th bit (hw_enc & 31) to get the actual hardware
    // register encoding.
    let preg = PReg::new(31 + 32, RegClass::Int);
    Reg::from(VReg::new(preg.index(), RegClass::Int))
}

/// Get a writable reference to the stack-pointer register.
pub fn writable_stack_reg() -> Writable<Reg> {
    Writable::from_reg(stack_reg())
}

/// Get a reference to the link register (x30).
pub fn link_reg() -> Reg {
    xreg(30)
}

/// Get a writable reference to the link register.
pub fn writable_link_reg() -> Writable<Reg> {
    Writable::from_reg(link_reg())
}

/// Get a reference to the frame pointer (x29).
pub fn fp_reg() -> Reg {
    xreg(29)
}

/// Get a writable reference to the frame pointer.
pub fn writable_fp_reg() -> Writable<Reg> {
    Writable::from_reg(fp_reg())
}

/// Get a reference to the first temporary, sometimes "spill temporary", register. This register is
/// used to compute the address of a spill slot when a direct offset addressing mode from FP is not
/// sufficient (+/- 2^11 words). We exclude this register from regalloc and reserve it for this
/// purpose for simplicity; otherwise we need a multi-stage analysis where we first determine how
/// many spill slots we have, then perhaps remove the reg from the pool and recompute regalloc.
///
/// We use x16 for this (aka IP0 in the AArch64 ABI) because it's a scratch register but is
/// slightly special (used for linker veneers). We're free to use it as long as we don't expect it
/// to live through call instructions.
pub fn spilltmp_reg() -> Reg {
    xreg(16)
}

/// Get a writable reference to the spilltmp reg.
pub fn writable_spilltmp_reg() -> Writable<Reg> {
    Writable::from_reg(spilltmp_reg())
}

/// Get a reference to the second temp register. We need this in some edge cases
/// where we need both the spilltmp and another temporary.
///
/// We use x17 (aka IP1), the other "interprocedural"/linker-veneer scratch reg that is
/// free to use otherwise.
pub fn tmp2_reg() -> Reg {
    xreg(17)
}

/// Get a writable reference to the tmp2 reg.
pub fn writable_tmp2_reg() -> Writable<Reg> {
    Writable::from_reg(tmp2_reg())
}

/// Create the register universe for AArch64.
pub fn create_reg_env(flags: &settings::Flags) -> MachineEnv {
    fn preg(r: Reg) -> PReg {
        r.to_real_reg().unwrap().into()
    }

    let mut env = MachineEnv {
        preferred_regs_by_class: [
            vec![
                preg(xreg(0)),
                preg(xreg(1)),
                preg(xreg(2)),
                preg(xreg(3)),
                preg(xreg(4)),
                preg(xreg(5)),
                preg(xreg(6)),
                preg(xreg(7)),
                preg(xreg(8)),
                preg(xreg(9)),
                preg(xreg(10)),
                preg(xreg(11)),
                preg(xreg(12)),
                preg(xreg(13)),
                preg(xreg(14)),
                preg(xreg(15)),
                // x16 and x17 are spilltmp and tmp2 (see above).
                // x18 could be used by the platform to carry inter-procedural state;
                // conservatively assume so and make it not allocatable.
                // x19-28 are callee-saved and so not preferred.
                // x21 is the pinned register (if enabled) and not allocatable if so.
                // x29 is FP, x30 is LR, x31 is SP/ZR.
            ],
            vec![
                preg(vreg(0)),
                preg(vreg(1)),
                preg(vreg(2)),
                preg(vreg(3)),
                preg(vreg(4)),
                preg(vreg(5)),
                preg(vreg(6)),
                preg(vreg(7)),
                // v8-15 are callee-saved and so not preferred.
                preg(vreg(16)),
                preg(vreg(17)),
                preg(vreg(18)),
                preg(vreg(19)),
                preg(vreg(20)),
                preg(vreg(21)),
                preg(vreg(22)),
                preg(vreg(23)),
                preg(vreg(24)),
                preg(vreg(25)),
                preg(vreg(26)),
                preg(vreg(27)),
                preg(vreg(28)),
                preg(vreg(29)),
                preg(vreg(30)),
                preg(vreg(31)),
            ],
        ],
        non_preferred_regs_by_class: [
            vec![
		// HFI: remove x19 and x20 from consideration to
		// emulate increased register pressure. The difference
		// between *this* and a normal configuration should
		// approximate the difference between a normal
		// configuration and a hypothetical one with two more
		// registers to play with, due to the lack of
		// long-lived values for heap base and limit.
                //preg(xreg(19)),
                //preg(xreg(20)),
                // x21 is pinned reg if enabled; we add to this list below if not.
                preg(xreg(22)),
                preg(xreg(23)),
                preg(xreg(24)),
                preg(xreg(25)),
                preg(xreg(26)),
                preg(xreg(27)),
                preg(xreg(28)),
            ],
            vec![
                preg(vreg(8)),
                preg(vreg(9)),
                preg(vreg(10)),
                preg(vreg(11)),
                preg(vreg(12)),
                preg(vreg(13)),
                preg(vreg(14)),
                preg(vreg(15)),
            ],
        ],
        fixed_stack_slots: vec![],
    };

    if !flags.enable_pinned_reg() {
        debug_assert_eq!(PINNED_REG, 21); // We assumed this above in hardcoded reg list.
        env.non_preferred_regs_by_class[0].push(preg(xreg(PINNED_REG)));
    }

    env
}

// PrettyPrint cannot be implemented for Reg; we need to invoke
// backend-specific functions from higher level (inst, arg, ...)
// types.

fn show_ireg(reg: RealReg) -> String {
    match reg.hw_enc() {
        29 => "fp".to_string(),
        30 => "lr".to_string(),
        31 => "xzr".to_string(),
        63 => "sp".to_string(),
        x => {
            debug_assert!(x < 29);
            format!("x{}", x)
        }
    }
}

fn show_vreg(reg: RealReg) -> String {
    format!("v{}", reg.hw_enc() & 31)
}

fn show_reg(reg: Reg) -> String {
    if let Some(rreg) = reg.to_real_reg() {
        match rreg.class() {
            RegClass::Int => show_ireg(rreg),
            RegClass::Float => show_vreg(rreg),
        }
    } else {
        format!("%{:?}", reg)
    }
}

pub fn pretty_print_reg(reg: Reg, allocs: &mut AllocationConsumer<'_>) -> String {
    let reg = allocs.next(reg);
    show_reg(reg)
}

/// If `ireg` denotes an Int-classed reg, make a best-effort attempt to show
/// its name at the 32-bit size.
pub fn show_ireg_sized(reg: Reg, size: OperandSize) -> String {
    let mut s = show_reg(reg);
    if reg.class() != RegClass::Int || !size.is32() {
        // We can't do any better.
        return s;
    }

    // Change (eg) "x42" into "w42" as appropriate
    if reg.class() == RegClass::Int && size.is32() && s.starts_with("x") {
        s = "w".to_string() + &s[1..];
    }

    s
}

/// Show a vector register used in a scalar context.
pub fn show_vreg_scalar(reg: Reg, size: ScalarSize) -> String {
    let mut s = show_reg(reg);
    if reg.class() != RegClass::Float {
        // We can't do any better.
        return s;
    }

    // Change (eg) "v0" into "d0".
    if s.starts_with("v") {
        let replacement = match size {
            ScalarSize::Size8 => "b",
            ScalarSize::Size16 => "h",
            ScalarSize::Size32 => "s",
            ScalarSize::Size64 => "d",
            ScalarSize::Size128 => "q",
        };
        s.replace_range(0..1, replacement);
    }

    s
}

/// Show a vector register.
pub fn show_vreg_vector(reg: Reg, size: VectorSize) -> String {
    assert_eq!(RegClass::Float, reg.class());
    let mut s = show_reg(reg);

    let suffix = match size {
        VectorSize::Size8x8 => ".8b",
        VectorSize::Size8x16 => ".16b",
        VectorSize::Size16x4 => ".4h",
        VectorSize::Size16x8 => ".8h",
        VectorSize::Size32x2 => ".2s",
        VectorSize::Size32x4 => ".4s",
        VectorSize::Size64x2 => ".2d",
    };

    s.push_str(suffix);
    s
}

/// Show an indexed vector element.
pub fn show_vreg_element(reg: Reg, idx: u8, size: ScalarSize) -> String {
    assert_eq!(RegClass::Float, reg.class());
    let s = show_reg(reg);
    let suffix = match size {
        ScalarSize::Size8 => ".b",
        ScalarSize::Size16 => ".h",
        ScalarSize::Size32 => ".s",
        ScalarSize::Size64 => ".d",
        _ => panic!("Unexpected vector element size: {:?}", size),
    };
    format!("{}{}[{}]", s, suffix, idx)
}

pub fn pretty_print_ireg(
    reg: Reg,
    size: OperandSize,
    allocs: &mut AllocationConsumer<'_>,
) -> String {
    let reg = allocs.next(reg);
    show_ireg_sized(reg, size)
}

pub fn pretty_print_vreg_scalar(
    reg: Reg,
    size: ScalarSize,
    allocs: &mut AllocationConsumer<'_>,
) -> String {
    let reg = allocs.next(reg);
    show_vreg_scalar(reg, size)
}

pub fn pretty_print_vreg_vector(
    reg: Reg,
    size: VectorSize,
    allocs: &mut AllocationConsumer<'_>,
) -> String {
    let reg = allocs.next(reg);
    show_vreg_vector(reg, size)
}

pub fn pretty_print_vreg_element(
    reg: Reg,
    idx: usize,
    size: ScalarSize,
    allocs: &mut AllocationConsumer<'_>,
) -> String {
    let reg = allocs.next(reg);
    show_vreg_element(reg, idx as u8, size)
}
