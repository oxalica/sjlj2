use super::NonZero;

// si, sp, bp, lander
#[repr(transparent)]
pub(crate) struct Buf(pub [usize; 4]);

macro_rules! set_jump_raw {
    ($buf_ptr:expr, $func:expr, $lander:block) => {
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
pub(crate) unsafe fn long_jump_raw(buf: *mut (), result: NonZero<usize>) -> ! {
    unsafe {
        maybe_strip_cfi!(
            (core::arch::asm!),

            [".cfi_remember_state"],
            [".cfi_undefined eip"],
            "mov esi, [ecx]",
            "mov [ecx], eax",
            "mov esp, [ecx + 4]",
            "mov ebp, [ecx + 8]",
            "jmp dword ptr [ecx + 12]",
            [".cfi_restore_state"],
            [],

            in("cx") buf,
            in("ax") result.get(),
            options(noreturn, nostack),
        )
    }
}
