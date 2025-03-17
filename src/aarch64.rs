use super::NonZero;

#[cfg(not(target_os = "macos"))]
macro_rules! set_jump_raw_impl {
    ($($tt:tt)*) => {
        core::arch::asm!(
            $($tt)*

            // Callee saved registers.
            // lateout("sp") _, // sp
            lateout("x16") _,
            lateout("x17") _,
            lateout("x18") _,
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
            // lateout("x29") _, // LLVM reserved.
            lateout("lr") _,
            // Caller saved registers.
            clobber_abi("C"),
        )
    }
}

#[cfg(all(target_os = "macos", feature = "unstable-asm-goto"))]
compile_error!(
    "aarch64-apple-darwin with feature 'unstable-asm-goto' has known miscompilation bug \
    caused by unrespected clobbered registers. More investigation is required. \
    Please disable the feature for now."
);

#[cfg(target_os = "macos")]
macro_rules! set_jump_raw_impl {
    ($($tt:tt)*) => {
        core::arch::asm!(
            $($tt)*

            // Callee saved registers.
            // lateout("sp") _, // sp
            lateout("x16") _,
            lateout("x17") _,
            // lateout("x18") _, // Macos reserved.
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
            // lateout("x29") _, // LLVM reserved.
            lateout("lr") _,
            // Caller saved registers.
            clobber_abi("C"),
        )
    }
}

macro_rules! set_jump_raw {
    ($val:expr, $f:expr, $data:expr, $lander:block) => {
        set_jump_raw_impl!(
            "adr x2, {lander}",
            "stp x2, x2, [sp, #-16]!",
            ".cfi_adjust_cfa_offset 16",
            "stp x19, x29, [sp, #-16]!",
            ".cfi_adjust_cfa_offset 16",
            "mov x1, sp",
            "bl {f}",
            "add sp, sp, 32",
            ".cfi_adjust_cfa_offset -32",

            f = sym $f,
            lander = label $lander,
            inout("x0") $data => $val,
        )
    };
    ($val:expr, $f:expr, $data:expr) => {
        set_jump_raw_impl!(
            "adr x2, 2f",
            "stp x2, x2, [sp, #-16]!",
            ".cfi_adjust_cfa_offset 16",
            "stp x19, x29, [sp, #-16]!",
            ".cfi_adjust_cfa_offset 16",
            "mov x1, sp",
            "bl {f}",
            "add sp, sp, 32",
            ".cfi_adjust_cfa_offset -32",
            "2:",

            f = sym $f,
            inout("x0") $data => $val,
        )
    };
}

#[inline]
pub(crate) unsafe fn long_jump_raw(jp: *mut (), result: NonZero<usize>) -> ! {
    unsafe {
        core::arch::asm!(
            "ldp x19, x29, [x1]",
            "ldr x2, [x1, #16]",
            "add sp, x1, 32",
            "br x2",
            in("x0") result.get(),
            in("x1") jp,
            options(noreturn, nostack, readonly),
        )
    }
}
