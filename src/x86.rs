use super::NonZero;

pub(crate) type Buf = [*mut (); 4];

macro_rules! set_jump_raw {
    ($val:expr, $buf_ptr:expr, $lander:block) => {
        core::arch::asm!(
            "lea eax, [{lander}]",
            "mov dword ptr [ecx], eax",
            "mov dword ptr [ecx + 4], esp",
            "mov dword ptr [ecx + 8], esi",
            "mov dword ptr [ecx + 12], ebp",
            in("cx") $buf_ptr, // Restored in long_jump_raw.
            lateout("ax") $val,
            lander = label $lander,

            // Callee saved registers.
            // lateout("si") _, // LLVM reserved.
            // lateout("sp") _, // sp
            // lateout("bp") _, // LLVM reserved.
            lateout("bx") _,
            lateout("di") _,
            // Caller saved registers.
            clobber_abi("C"),
            options(readonly),
        )
    };
    ($val:expr, $buf_ptr:expr) => {
        core::arch::asm!(
            "lea eax, [3f]",
            "mov dword ptr [ecx], eax",
            "mov dword ptr [ecx + 4], esp",
            "mov dword ptr [ecx + 8], esi",
            "mov dword ptr [ecx + 12], ebp",
            "xor eax, eax",
            "3:",
            in("cx") $buf_ptr, // Restored in long_jump_raw.
            lateout("ax") $val,

            // Callee saved registers.
            // lateout("si") _, // LLVM reserved.
            // lateout("sp") _, // sp
            // lateout("bp") _, // LLVM reserved.
            lateout("bx") _,
            lateout("di") _,
            // Caller saved registers.
            clobber_abi("C"),
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
            "mov ebp, dword ptr [ecx + 12]",
            "mov esi, dword ptr [ecx + 8]",
            "mov esp, dword ptr [ecx + 4]",
            "jmp dword ptr [ecx]",
            ".cfi_restore_state",
            in("cx") buf,
            in("ax") result.get(),
            options(noreturn),
        )
    }
}
