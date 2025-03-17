use super::NonZero;

macro_rules! set_jump_raw {
    ($val:expr, $f:expr, $data:expr, $lander:block) => {
        core::arch::asm!(
            "lea rax, [rip + {lander}]",
            "mov rsi, rsp",
            "push rax",
            ".cfi_adjust_cfa_offset 8",
            "push rax", // Align the stack.
            ".cfi_adjust_cfa_offset 8",
            "push rbx",
            ".cfi_adjust_cfa_offset 8",
            "push rbp",
            ".cfi_adjust_cfa_offset 8",
            "call {f}",
            "add rsp, 32",
            ".cfi_adjust_cfa_offset -32",
            f = sym $f,
            in("rdi") $data,
            out("rax") $val,
            lander = label $lander,

            // Callee saved registers.
            // lateout("bx") _, // LLVM reserved.
            // lateout("sp") _, // sp
            // lateout("bp") _, // LLVM reserved.
            lateout("r12") _,
            lateout("r13") _,
            lateout("r14") _,
            lateout("r15") _,
            // Caller saved registers.
            clobber_abi("sysv64"),
        )
    };
    ($val:expr, $f:expr, $data:expr) => {
        core::arch::asm!(
            "push rbx",
            ".cfi_adjust_cfa_offset 8",
            "push rbp",
            ".cfi_adjust_cfa_offset 8",
            "mov rsi, rsp",
            "call {f}",
            "add rsp, 16",
            ".cfi_adjust_cfa_offset -16",
            f = sym $f,
            in("rdi") $data,
            out("rax") $val,

            // Callee saved registers.
            // lateout("bx") _, // LLVM reserved.
            // lateout("sp") _, // sp
            // lateout("bp") _, // LLVM reserved.
            lateout("r12") _,
            lateout("r13") _,
            lateout("r14") _,
            lateout("r15") _,
            // Caller saved registers.
            clobber_abi("sysv64"),
        )
    };
}

#[inline]
pub(crate) unsafe fn long_jump_raw(jp: *mut (), result: NonZero<usize>) -> ! {
    #[cfg(feature = "unstable-asm-goto")]
    unsafe {
        core::arch::asm!(
            "mov rdx, qword ptr [rcx - 16]",
            "mov rbx, qword ptr [rcx - 24]",
            "mov rbp, qword ptr [rcx - 32]",
            "mov rsp, rcx",
            "jmp rdx",
            in("cx") jp,
            in("ax") result.get(),
            options(noreturn, nostack, readonly),
        )
    }

    #[cfg(not(feature = "unstable-asm-goto"))]
    unsafe {
        core::arch::asm!(
            "mov rdx, qword ptr [rcx - 8]",
            "mov rbp, qword ptr [rcx]",
            "mov rbx, qword ptr [rcx + 8]",
            "mov rsp, rcx",
            "jmp rdx",
            in("cx") jp,
            in("ax") result.get(),
            options(noreturn, nostack, readonly),
        )
    }
}
