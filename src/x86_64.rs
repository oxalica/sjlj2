// rbx, rsp, rbp, lander
#[repr(transparent)]
pub(crate) struct Buf(pub [usize; 4]);

macro_rules! set_jump_raw {
    ($buf_ptr:expr, $func:expr, $lander:block) => {
        core::arch::asm!(
            "lea rax, [rip + {lander}]",
            "mov [rdi     ], rbx",
            "mov [rdi +  8], rsp",
            "mov [rdi + 16], rbp",
            "mov [rdi + 24], rax",
            "call {func}",

            in("rdi") $buf_ptr, // arg0
            func = sym $func,
            lander = label $lander,

            // Clobber more default callee saved registers.
            // lateout("bx") _, // LLVM reserved.
            // lateout("sp") _, // sp
            // lateout("bp") _, // LLVM reserved.
            lateout("r12") _,
            lateout("r13") _,
            lateout("r14") _,
            lateout("r15") _,

            // Default caller saved registers.
            clobber_abi("sysv64"),
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
            ".cfi_undefined rip",
            "mov rbx, [rcx     ]",
            "mov rsp, [rcx +  8]",
            "mov rbp, [rcx + 16]",
            "mov [rcx], rax",
            "jmp qword ptr [rcx + 24]",
            #[cfg(emit_cfi)]
            ".cfi_restore_state",

            in("cx") jp,
            in("ax") data,
            options(noreturn, nostack),
        )
    }
}
