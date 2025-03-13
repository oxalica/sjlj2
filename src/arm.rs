use super::NonZero;

pub(crate) type Buf = [*mut (); 4];

macro_rules! set_jump_raw {
    ($val:expr, $buf_ptr:expr, $lander:block) => {
        core::arch::asm!(
            "adr lr, {lander}",
            "stm r1, {{r6, r11, sp, lr}}",

            lander = label $lander,

            out("r0") $val,
            in("r1") $buf_ptr, // Restored in long_jump_raw.

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
            clobber_abi("aapcs"),
            options(readonly),
        )
    };
    ($val:expr, $buf_ptr:expr) => {
        core::arch::asm!(
            "adr lr, 2f",
            "stm r1, {{r6, r11, sp, lr}}",
            "mov r0, 0",
            "2:",

            // Caller saved registers.
            // Workaround: see above.
            out("r0") $val,
            in("r1") $buf_ptr, // Restored in long_jump_raw.

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
            clobber_abi("aapcs"),
            options(readonly),
        )
    };
}

#[inline]
pub(crate) unsafe fn long_jump_raw(buf: *mut (), result: NonZero<usize>) -> ! {
    unsafe {
        core::arch::asm!(
            "ldm r1, {{r6, r11, sp, pc}}",
            in("r1") buf,
            in("r0") result.get(),
            options(noreturn),
        )
    }
}
