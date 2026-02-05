// si, sp, bp, lander
#[repr(transparent)]
pub(crate) struct Buf(pub [usize; 4]);

macro_rules! set_jump_raw {
    ($buf_ptr:expr, $func:path, $lander:block) => {
        core::arch::asm!(
            "call 2f",
            "2:",
            "popl %eax",
            "addl $({lander} - 2b), %eax",
            "movl %esi,   (%ecx)",
            "movl %esp,  4(%ecx)",
            "movl %ebp,  8(%ecx)",
            "movl %eax, 12(%ecx)",
            "call {func}",

            in("cx") $buf_ptr, // arg0 for fastcall
            func = sym $func,
            lander = label $lander,
            // Workaround: <https://github.com/rust-lang/rust/issues/74558>
            options(att_syntax),

            // Callee saved registers.
            // lateout("si") _, // LLVM reserved.
            // lateout("sp") _, // sp
            // lateout("bp") _, // LLVM reserved.
            lateout("bx") _,
            lateout("di") _,
            // Caller saved registers.
            clobber_abi("fastcall"),
        )
    };
}

#[inline]
pub(crate) unsafe fn long_jump_raw(buf: *mut (), data: usize) -> ! {
    unsafe {
        core::arch::asm!(
            #[cfg(emit_cfi)]
            ".cfi_remember_state",
            #[cfg(emit_cfi)]
            ".cfi_undefined eip",

            "mov esi, [ecx]",
            "mov [ecx], eax",
            "mov esp, [ecx + 4]",
            "mov ebp, [ecx + 8]",
            "jmp dword ptr [ecx + 12]",

            #[cfg(emit_cfi)]
            ".cfi_restore_state",

            in("cx") buf,
            in("ax") data,
            options(noreturn, nostack),
        )
    }
}
