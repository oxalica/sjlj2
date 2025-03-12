use super::*;

pub(crate) type Buf = [*mut (); 4];

macro_rules! set_jump_raw {
    ($val:expr, $buf_ptr:expr, $lander:block) => {
        core::arch::asm!(
            "la a0, {lander}",
            "sd a0, 0(a1)",
            "sd sp, 8(a1)",
            "sd s0, 16(a1)",
            "sd s1, 24(a1)",

            in("a1") $buf_ptr, // Restored in long_jump_raw.
            out("a0") $val,
            lander = label $lander,

            // Callee saved registers.
            lateout("ra") _,
            // lateout("sp") _, // sp
            // lateout("s0") _, // LLVM reserved.
            // lateout("s1") _, // LLVM reserved.
            lateout("s2") _,
            lateout("s3") _,
            lateout("s4") _,
            lateout("s5") _,
            lateout("s6") _,
            lateout("s7") _,
            lateout("s8") _,
            lateout("s9") _,
            lateout("s10") _,
            lateout("s11") _,
            lateout("fs0") _,
            lateout("fs1") _,
            lateout("fs2") _,
            lateout("fs3") _,
            lateout("fs4") _,
            lateout("fs5") _,
            lateout("fs6") _,
            lateout("fs7") _,
            lateout("fs8") _,
            lateout("fs9") _,
            lateout("fs10") _,
            lateout("fs11") _,
            // Caller saved registers.
            clobber_abi("C"),
            options(readonly),
        )
    };
    ($val:expr, $buf_ptr:expr) => {
        core::arch::asm!(
            "la a0, 2f",
            "sd a0, 0(a1)",
            "sd sp, 8(a1)",
            "sd s0, 16(a1)",
            "sd s1, 24(a1)",
            "li a0, 0",
            "2:",

            in("a1") $buf_ptr, // Restored in long_jump_raw.
            out("a0") $val,

            // Callee saved registers.
            lateout("ra") _,
            // lateout("sp") _, // sp
            // lateout("s0") _, // LLVM reserved.
            // lateout("s1") _, // LLVM reserved.
            lateout("s2") _,
            lateout("s3") _,
            lateout("s4") _,
            lateout("s5") _,
            lateout("s6") _,
            lateout("s7") _,
            lateout("s8") _,
            lateout("s9") _,
            lateout("s10") _,
            lateout("s11") _,
            lateout("fs0") _,
            lateout("fs1") _,
            lateout("fs2") _,
            lateout("fs3") _,
            lateout("fs4") _,
            lateout("fs5") _,
            lateout("fs6") _,
            lateout("fs7") _,
            lateout("fs8") _,
            lateout("fs9") _,
            lateout("fs10") _,
            lateout("fs11") _,
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
            "ld ra, 0(a1)",
            "ld sp, 8(a1)",
            "ld s0, 16(a1)",
            "ld s1, 24(a1)",
            "jalr x0, ra",
            in("a1") buf,
            in("a0") result.get(),
            options(noreturn),
        )
    }
}
