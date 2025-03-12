use super::NonZero;

pub(crate) type Buf = [*mut (); 4];

macro_rules! set_jump_raw {
    ($val:expr, $buf_ptr:expr, $lander:block) => {
        core::arch::asm!(
            "lea rax, [rip + {lander}]",
            "mov qword ptr [rcx], rax",
            "mov qword ptr [rcx + 8], rsp",
            "mov qword ptr [rcx + 16], rbx",
            "mov qword ptr [rcx + 24], rbp",
            in("cx") $buf_ptr, // Restored in long_jump_raw.
            lateout("ax") $val,
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
            options(readonly),
        )
    };
    ($val:expr, $buf_ptr:expr) => {
        core::arch::asm!(
            "lea rax, [rip + 2f]",
            "mov qword ptr [rcx], rax",
            "mov qword ptr [rcx + 8], rsp",
            "mov qword ptr [rcx + 16], rbx",
            "mov qword ptr [rcx + 24], rbp",
            "xor eax, eax",
            "2:",
            in("cx") $buf_ptr, // Restored in long_jump_raw.
            lateout("ax") $val,

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
            options(readonly),
        )
    };
}

#[inline]
pub(crate) unsafe fn long_jump_raw(buf: *mut (), result: NonZero<usize>) -> ! {
    unsafe {
        core::arch::asm!(
            ".cfi_remember_state",
            ".cfi_undefined",
            "mov rbp, qword ptr [rcx + 24]",
            "mov rbx, qword ptr [rcx + 16]",
            "mov rsp, qword ptr [rcx + 8]",
            "jmp qword ptr [rcx]",
            ".cfi_restore_state",
            in("cx") buf,
            in("ax") result.get(),
            options(noreturn),
        )
    }
}
