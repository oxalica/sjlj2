use super::NonZero;

macro_rules! set_jump_raw_impl {
    ($($tt:tt)*) => {
        maybe_strip_cfi!(
            (core::arch::asm!),
            $($tt)*

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
    }
}

macro_rules! set_jump_raw {
    ($val:expr, $f:expr, $data:expr, $lander:block) => {
        set_jump_raw_impl!(
            "adr lr, {lander}",
            "push {{r6, r11, sp, lr}}",
            [".cfi_adjust_cfa_offset 16"],
            "mov r1, sp",
            "bl {f}",
            "add sp, sp, 16",
            [".cfi_adjust_cfa_offset -16"],
            [],

            f = sym $f,
            inout("r0") $data => $val,
            lander = label $lander,
        )
    };
    ($val:expr, $f:expr, $data:expr) => {
        set_jump_raw_impl!(
            "adr lr, 2f",
            "push {{r6, r11, sp, lr}}",
            [".cfi_adjust_cfa_offset 16"],
            "mov r1, sp",
            "bl {f}",
            "add sp, sp, 16",
            [".cfi_adjust_cfa_offset -16"],
            "2:",
            [],

            f = sym $f,
            inout("r0") $data => $val,
        )
    };
}

#[inline]
pub(crate) unsafe fn long_jump_raw(buf: *mut (), result: NonZero<usize>) -> ! {
    unsafe {
        core::arch::asm!(
            "ldm r1, {{r6, r11, sp, pc}}",
            in("r0") result.get(),
            in("r1") buf,
            options(noreturn, nostack, readonly),
        )
    }
}
