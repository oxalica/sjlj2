use super::NonZero;

macro_rules! set_jump_raw_impl {
    ($($tt:tt)*) => {
        core::arch::asm!(
            $($tt)*

            // Callee saved registers.
            lateout("ra") _,
            // lateout("sp") _, // sp
            // lateout("s0") _, // LLVM reserved.
            // lateout("s1") _, // LLVM reserved.
            lateout("s2") _,
            lateout("s3") _,
            lateout("s4") _,
            lateout("s5") _,
            lateout("s6") _,
            lateout("s7") _,
            lateout("s8") _,
            lateout("s9") _,
            lateout("s10") _,
            lateout("s11") _,
            lateout("fs0") _,
            lateout("fs1") _,
            lateout("fs2") _,
            lateout("fs3") _,
            lateout("fs4") _,
            lateout("fs5") _,
            lateout("fs6") _,
            lateout("fs7") _,
            lateout("fs8") _,
            lateout("fs9") _,
            lateout("fs10") _,
            lateout("fs11") _,
            // Caller saved registers.
            clobber_abi("C"),
        )
    };
}

macro_rules! set_jump_raw {
    ($val:expr, $f:expr, $data:expr, $lander:block) => {
        set_jump_raw_impl!(
            "la a1, {lander}",
            "addi sp, sp, -32",
            ".cfi_adjust_cfa_offset 32",
            "sd a1, (sp)",
            "sd s0, 8(sp)",
            "sd s1, 16(sp)",
            "mv a1, sp",
            "call {f}",
            "addi sp, sp, 32",
            ".cfi_adjust_cfa_offset -32",

            f = sym $f,
            inout("a0") $data => $val,
            lander = label $lander,
        )
    };
    ($val:expr, $f:expr, $data:expr) => {
        set_jump_raw_impl!(
            "la a1, 2f",
            "addi sp, sp, -32",
            ".cfi_adjust_cfa_offset 32",
            "sd a1, (sp)",
            "sd s0, 8(sp)",
            "sd s1, 16(sp)",
            "mv a1, sp",
            "call {f}",
            "addi sp, sp, 32",
            ".cfi_adjust_cfa_offset -32",
            "2:",

            f = sym $f,
            inout("a0") $data => $val,
        )
    };
}

#[inline]
pub(crate) unsafe fn long_jump_raw(jp: *mut (), result: NonZero<usize>) -> ! {
    unsafe {
        core::arch::asm!(
            "ld a2, 0(a1)",
            "ld s0, 8(a1)",
            "ld s1, 16(a1)",
            "addi sp, a1, 32",
            "jalr x0, a2",
            in("a0") result.get(),
            in("a1") jp,
            options(noreturn, nostack, readonly),
        )
    }
}
