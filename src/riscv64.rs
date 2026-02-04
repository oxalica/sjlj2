// s0, s1, sp, lander
#[repr(transparent)]
pub(crate) struct Buf(pub [usize; 4]);

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
    ($buf_ptr:expr, $func:path, $lander:block) => {
        set_jump_raw_impl!(
            "la a1, {lander}",
            "sd s0,   (a0)",
            "sd s1,  8(a0)",
            "sd sp, 16(a0)",
            "sd a1, 24(a0)",
            "call {func}",

            in("a0") $buf_ptr, // arg0
            func = sym $func,
            lander = label $lander,
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

            "ld s0,   (a0)",
            "sd a1,   (a0)",
            "ld s1,  8(a0)",
            "ld sp, 16(a0)",
            "ld a2, 24(a0)",
            "jalr x0, a2",

            #[cfg(emit_cfi)]
            ".cfi_restore_state",

            in("a0") jp,
            in("a1") data,
            options(noreturn, nostack),
        )
    }
}
