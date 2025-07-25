use super::NonZero;

// result, r6, r11, sp, lander
pub(crate) struct Buf(pub [usize; 5]);

macro_rules! set_jump_raw {
    ($buf_ptr:expr, $func:expr, $lander:block) => {
        core::arch::asm!(
            "adr lr, {lander}",
            "stm r0, {{r0, r6, r11, sp, lr}}",
            "bl {func}",

            in("r0") $buf_ptr, // arg0
            func = sym $func,
            lander = label $lander,

            // Callee saved registers.
            lateout("r4") _,
            lateout("r5") _,
            // lateout("r6") _, // LLVM reserved.
            lateout("r7") _,
            lateout("r8") _,
            lateout("r9") _,
            lateout("r10") _,
            // lateout("r11") _, // LLVM reserved.
            lateout("r12") _,
            // lateout("sp") _, // sp
            lateout("lr") _,
            // Caller saved registers.
            // FIXME: inline asm clobber list contains reserved registers: D16-D31.
            clobber_abi("aapcs"),
        )
    };
}

#[inline]
pub(crate) unsafe fn long_jump_raw(buf: *mut (), result: NonZero<usize>) -> ! {
    unsafe {
        core::arch::asm!(
            "str r0, [r1]",
            "ldm r1, {{r0, r6, r11, sp, pc}}",
            in("r0") result.get(),
            in("r1") buf,
            options(noreturn, nostack, readonly),
        )
    }
}
