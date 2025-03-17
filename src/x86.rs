use super::NonZero;

macro_rules! set_jump_raw_impl {
    ($($tt:tt)*) => {
        core::arch::asm!(
            $($tt)*

            // Callee saved registers.
            // lateout("si") _, // LLVM reserved.
            // lateout("sp") _, // sp
            // lateout("bp") _, // LLVM reserved.
            lateout("bx") _,
            lateout("di") _,
            // Caller saved registers.
            clobber_abi("C"),
        )
    };
}

macro_rules! set_jump_raw {
    ($val:expr, $f:expr, $data:expr, $lander:block) => {
        set_jump_raw_impl!(
            "call 2f",
            "2:",
            ".cfi_adjust_cfa_offset 4",
            "addl $({lander} - 2b), (%esp)",
            ".cfi_adjust_cfa_offset 4",
            "pushl %esi",
            ".cfi_adjust_cfa_offset 4",
            "pushl %ebp",
            ".cfi_adjust_cfa_offset 4",
            "pushl %esp",
            ".cfi_adjust_cfa_offset 4",
            "pushl {data}",
            ".cfi_adjust_cfa_offset 4",
            "call {f}",
            "addl $20, %esp",
            ".cfi_adjust_cfa_offset -20",

            f = sym $f,
            data = in(reg) $data,
            out("ax") $val,
            lander = label $lander,
            // Workaround: <https://github.com/rust-lang/rust/issues/74558>
            options(att_syntax),
        )
    };
    ($val:expr, $f:expr, $data:expr) => {
        set_jump_raw_impl!(
            "call 2f",
            "2:",
            ".cfi_adjust_cfa_offset 4",
            "addl $(3f - 2b), (%esp)",
            "pushl %esi",
            ".cfi_adjust_cfa_offset 4",
            "pushl %ebp",
            ".cfi_adjust_cfa_offset 4",
            "pushl %esp",
            ".cfi_adjust_cfa_offset 4",
            "pushl {data}",
            ".cfi_adjust_cfa_offset 4",
            "call {f}",
            "addl $20, %esp",
            ".cfi_adjust_cfa_offset -20",
            "3:",

            f = sym $f,
            data = in(reg) $data,
            out("ax") $val,
            // Workaround: <https://github.com/rust-lang/rust/issues/74558>
            options(att_syntax),
        )
    };
}

#[inline]
pub(crate) unsafe fn long_jump_raw(buf: *mut (), result: NonZero<usize>) -> ! {
    unsafe {
        core::arch::asm!(
            "mov edx, dword ptr [ecx - 4]",
            "mov esi, dword ptr [ecx - 8]",
            "mov ebp, dword ptr [ecx - 12]",
            "mov esp, ecx",
            "jmp edx",
            in("cx") buf as usize + 12,
            in("ax") result.get(),
            options(noreturn, nostack, readonly),
        )
    }
}
