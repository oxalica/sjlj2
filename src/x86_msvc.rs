//! i686-pc-windows-msvc needs special handling because we need to
//! save/restore the runtime SEH chain.
//! Otherwise, `long_jmp` from `catch_unwind` in the ordinary path will leave
//! the SEH chain un-restored, causing any later exception segfaults.

// si, sp, bp, lander, SEH head
#[repr(transparent)]
pub(crate) struct Buf(pub [usize; 5]);

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
            "movl %fs:0, %eax",
            "movl %eax, 16(%ecx)",
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
            "mov eax, [ecx + 16]",
            "mov fs:[0], eax",
            "jmp dword ptr [ecx + 12]",

            #[cfg(emit_cfi)]
            ".cfi_restore_state",

            in("cx") buf,
            in("ax") data,
            options(noreturn, nostack),
        )
    }
}
