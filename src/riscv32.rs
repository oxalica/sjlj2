// s0, s1, sp, lander
#[repr(transparent)]
pub(crate) struct Buf(pub [usize; 4]);

macro_rules! set_jump_raw {
    ($buf_ptr:expr, $func:path, $lander:block) => {
        core::arch::asm!(
            "la a1, {lander}",
            "sw s0,   (a0)",
            "sw s1,  4(a0)",
            "sw sp,  8(a0)",
            "sw a1, 12(a0)",
            "call {func}",

            in("a0") $buf_ptr, // arg0
            func = sym $func,
            lander = label $lander,

            // Callee saved registers.
            lateout("ra") _,
            // lateout("sp") _, // sp
            // lateout("s0") _, // LLVM reserved.
            // lateout("s1") _, // LLVM reserved.
            #[cfg(not(target_feature = "e"))]
            lateout("s2") _,
            #[cfg(not(target_feature = "e"))]
            lateout("s3") _,
            #[cfg(not(target_feature = "e"))]
            lateout("s4") _,
            #[cfg(not(target_feature = "e"))]
            lateout("s5") _,
            #[cfg(not(target_feature = "e"))]
            lateout("s6") _,
            #[cfg(not(target_feature = "e"))]
            lateout("s7") _,
            #[cfg(not(target_feature = "e"))]
            lateout("s8") _,
            #[cfg(not(target_feature = "e"))]
            lateout("s9") _,
            #[cfg(not(target_feature = "e"))]
            lateout("s10") _,
            #[cfg(not(target_feature = "e"))]
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

#[inline]
pub(crate) unsafe fn long_jump_raw(jp: *mut (), data: usize) -> ! {
    unsafe {
        core::arch::asm!(
            #[cfg(emit_cfi)]
            ".cfi_remember_state",
            #[cfg(emit_cfi)]
            ".cfi_undefined ra",

            "lw s0,   (a0)",
            "sw a1,   (a0)",
            "lw s1,  4(a0)",
            "lw sp,  8(a0)",
            "lw a2, 12(a0)",
            "jalr x0, a2",

            #[cfg(emit_cfi)]
            ".cfi_restore_state",

            in("a0") jp,
            in("a1") data,
            options(noreturn, nostack),
        )
    }
}
