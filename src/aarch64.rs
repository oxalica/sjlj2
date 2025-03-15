use super::NonZero;

pub(crate) type Buf = [*mut (); 4];

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
            options(readonly),
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
            options(readonly),
        )
    }
}

macro_rules! set_jump_raw {
    ($val:expr, $buf_ptr:expr, $lander:block) => {
        set_jump_raw_impl!(
            "adr x0, {lander}",
            "mov x2, sp",
            "stp x0, x2, [x1]",
            "stp x19, x29, [x1, #16]",

            in("x1") $buf_ptr, // Restored in long_jump_raw.
            out("x0") $val,
            lander = label $lander,
        )
    };
    ($val:expr, $buf_ptr:expr) => {
        set_jump_raw_impl!(
            "adr x0, 2f",
            "mov x2, sp",
            "stp x0, x2, [x1]",
            "stp x19, x29, [x1, #16]",
            "mov x0, 0",
            "2:",

            in("x1") $buf_ptr, // Restored in long_jump_raw.
            out("x0") $val,
        )
    };
}

#[inline]
pub(crate) unsafe fn long_jump_raw(buf: *mut (), result: NonZero<usize>) -> ! {
    unsafe {
        core::arch::asm!(
            "ldp x19, x29, [x1, #16]",
            "ldp lr, x2, [x1]",
            "mov sp, x2",
            "ret",
            in("x1") buf,
            in("x0") result.get(),
            options(noreturn),
        )
    }
}
