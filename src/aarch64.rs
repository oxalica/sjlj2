// sp, x19, fp, lander
#[repr(C, align(16))]
pub(crate) struct Buf(pub [usize; 4]);

// x18 is platform-reserved on darwin. Do not touch it at all.
// See: <https://stackoverflow.com/questions/71152539/consequence-of-violating-macoss-arm64-calling-convention>
#[cfg(target_os = "macos")]
macro_rules! set_jump_raw_impl {
    ($($tt:tt)*) => {
        core::arch::asm!($($tt)*)
    };
}

// On non-darwin platform, mark x18 clobbered.
#[cfg(not(target_os = "macos"))]
macro_rules! set_jump_raw_impl {
    ($($tt:tt)*) => {
        core::arch::asm!($($tt)* lateout("x18") _)
    };
}

macro_rules! set_jump_raw {
    ($buf_ptr:expr, $func:expr, $lander:block) => {
        set_jump_raw_impl!(
            "adr x1, {lander}",
            "mov x2, sp",
            "stp x2, x19, [x0]",
            "stp fp, x1, [x0, #16]",
            "bl {func}",

            in("x0") $buf_ptr, // arg0
            func = sym $func,
            lander = label $lander,

            // Callee saved registers.
            // lateout("sp") _, // sp
            lateout("x16") _,
            lateout("x17") _,
            // lateout("x18") _, // See `set_jump_raw_impl`
            // lateout("x19") _, // LLVM reserved.
            lateout("x20") _,
            lateout("x21") _,
            lateout("x22") _,
            lateout("x23") _,
            lateout("x24") _,
            lateout("x25") _,
            lateout("x26") _,
            lateout("x27") _,
            lateout("x28") _,
            // lateout("fp") _, // LLVM reserved.
            lateout("lr") _,
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
            ".cfi_undefined lr",

            "ldp x2, x19, [x1]",
            "ldp fp, lr, [x1, #16]",
            "mov sp, x2",
            "str x0, [x1]",
            "ret",

            #[cfg(emit_cfi)]
            ".cfi_restore_state",

            in("x0") data,
            in("x1") jp,
            options(noreturn, nostack),
        )
    }
}
