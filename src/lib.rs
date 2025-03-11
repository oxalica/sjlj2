//! # Safer[^1], cheaper and more ergonomic setjmp/longjmp in Rust[^2].
//!
//! [^1]: [`long_jump`] is still unsafe, though, due to [POF][pof] restrictions. See more in [`long_jump`].
//! [^2]: ...and assembly. No C trampoline is involved!
//!
//! [pof]: https://rust-lang.github.io/rfcs/2945-c-unwind-abi.html#plain-old-frames
//!
//! - Ergonomic Rusty API for typical usages. Uses closure API instead of multiple-return.
//!
//!   Multiple-return functions are undefined behaviors due to [bad (fatal) interaction with
//!   optimizer](https://github.com/rust-lang/rfcs/issues/2625).
//!
//! - Single-use jump checkpoint.
//!
//!   No jump-after-jump disaster. No coroutine-at-home. Less register backups.
//!
//! - Minimal memory and performance footprint.
//!
//!   Single `usize` state. Let optimizer save only necessary states rather than bulk saving.
#![cfg_attr(not(test), no_std)]
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::num::NonZero;

/// A jump checkpoint that you can go back to at any time.
///
/// It consists of a single machine word, the stack pointer.
#[doc(alias = "jmp_buf")]
#[derive(Debug, Clone, Copy)]
pub struct JumpPoint<'a>(*mut (), PhantomData<&'a mut &'a ()>);

#[cfg(doctest)]
/// ```compile_fail
/// fn f(j: sjlj2::JumpPoint<'_>) -> impl Send { j }
/// ```
///
/// ```compile_fail
/// fn f(j: sjlj2::JumpPoint<'_>) -> impl Sync { j }
/// ```
fn _assert_not_or_sync() {}

impl JumpPoint<'_> {
    /// Reconstruct from a raw state.
    ///
    /// # Safety
    ///
    /// `raw` must be a valid state returned [`JumpPoint::as_raw`], and the returned type must not
    /// outlive the lifetime of the original [`JumpPoint`] (that is, the closure `ordinary` of
    /// [`set_jump`]).
    pub const unsafe fn from_raw(raw: *mut ()) -> Self {
        Self(raw, PhantomData)
    }

    /// Get the underlying raw state.
    pub fn as_raw(&self) -> *mut () {
        self.0
    }
}

/// Set a jump checkpoint.
///
/// Set a jump checkpoint and execute `ordinary` with a checkpoint argument and return its result.
/// If a long jump is issued on the checkpoint inside execution of `ordinary`, control flow goes
/// back, execute `lander` and return its result from `set_jump`.
///
/// # Safety
///
/// Yes, this function is actually safe to use. [`long_jump`] is unsafe, however.
#[doc(alias = "setjmp")]
#[inline]
pub fn set_jump<T, F, G>(ordinary: F, lander: G) -> T
where
    F: FnOnce(JumpPoint<'_>) -> T,
    G: FnOnce(NonZero<usize>) -> T,
{
    union Arg<T, F> {
        f: ManuallyDrop<F>,
        ret: ManuallyDrop<T>,
    }

    extern "C" fn wrap<T, F: FnOnce(JumpPoint<'_>) -> T>(arg: *mut (), sp: *mut ()) {
        let arg = unsafe { &mut *arg.cast::<Arg<T, F>>() };
        let ret = unsafe { ManuallyDrop::take(&mut arg.f)(JumpPoint::from_raw(sp)) };
        arg.ret = ManuallyDrop::new(ret);
    }

    let mut arg = Arg::<T, F> {
        f: ManuallyDrop::new(ordinary),
    };
    let ret = unsafe { imp::set_jump_raw(wrap::<T, F>, (&raw mut arg).cast()) };
    match NonZero::new(ret) {
        None => unsafe { ManuallyDrop::take(&mut arg.ret) },
        Some(ret) => lander(ret),
    }
}

/// Long jump to a checkpoint, and executing corresponding `set_jump`'s `exceptional` closure.
///
/// # Safety
///
/// - You must not unwind across a `set_jump` boundary.
///   We will try our best to detect it and will abort the program in this case. But you must not
///   rely on it.
///
/// - When [`long_jump`] is called on the [`JumpPoint`] argument of `ordinary` closure during its
///   execution, all stack frames between that `long_jump` and this `set_jump`, must be all
///   [Plain Old Frames][pof].
///
/// [pof]: https://rust-lang.github.io/rfcs/2945-c-unwind-abi.html#plain-old-frames
#[doc(alias = "longjmp")]
#[inline]
pub unsafe fn long_jump(point: JumpPoint<'_>, result: NonZero<usize>) -> ! {
    unsafe { imp::long_jump_raw(point.0, result) }
}

#[cfg(all(target_arch = "x86_64", not(windows)))]
mod imp {
    use super::*;

    #[inline]
    pub(crate) unsafe fn set_jump_raw(
        f: extern "C" fn(arg: *mut (), sp: *mut ()),
        arg: *mut (),
    ) -> usize {
        let mut ret;
        unsafe {
            core::arch::asm!(
                "lea rsi, [rip + 2f]",
                "push rdi", // arg (align)
                "push rbx",
                "push rbp",
                "push rsi",
                "mov rsi, rsp",
                "call {f}",
                "add rsp, 32",
                "xor esi, esi",
                "jmp 3f",
                "2:",
                // Long jump lander.
                "mov rsp, rax",
                "pop rax", // lander rip
                "pop rbp",
                "pop rbx",
                "pop rdi", // arg (align)
                "3:",

                f = in(reg) f,
                in("di") arg,
                out("si") ret, // Do not use as input.

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
            );
        }
        ret
    }

    #[inline]
    pub(crate) unsafe fn long_jump_raw(sp: *mut (), result: NonZero<usize>) -> ! {
        unsafe {
            core::arch::asm!(
                "jmp qword ptr [rax]",
                in("ax") sp,
                in("si") result.get(),
                options(noreturn),
            )
        }
    }
}

#[cfg(not(all(target_arch = "x86_64", not(windows))))]
mod imp {
    use super::*;

    compile_error!("sjlj2: unsupported platform");

    #[inline]
    pub(crate) unsafe fn set_jump_raw(
        _f: extern "C" fn(arg: *mut (), sp: *mut ()),
        _arg: *mut (),
    ) -> usize {
        unimplemented!()
    }

    pub(crate) unsafe fn long_jump_raw(_sp: *mut (), _result: NonZero<usize>) -> ! {
        unimplemented!()
    }
}
