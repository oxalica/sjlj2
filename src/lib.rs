//! # Safer[^1], cheaper and more ergonomic setjmp/longjmp in Rust[^2].
//!
//! [^1]: [`long_jump`] is still unsafe, though, due to [POF][pof] restrictions.
//!       See more in [`long_jump`].
//! [^2]: ...and assembly. No C trampoline is involved!
//!
//! [pof]: https://rust-lang.github.io/rfcs/2945-c-unwind-abi.html#plain-old-frames
//!
//! - Ergonomic Rusty API for typical usages. Uses closure API instead of multiple-return.
//!
//!   Multiple-return functions are undefined behaviors due to
//!   [fatal interaction with optimizer](https://github.com/rust-lang/rfcs/issues/2625).
//!   This crate does not suffer from the misoptimization (covered in `tests/smoke.rs`).
//!
//! - Single-use jump checkpoint.
//!
//!   No jump-after-jump disaster. No coroutine-at-home.
//!
//! - Minimal memory and performance footprint.
//!
//!   Single `usize` `JumpPoint`. Let optimizer save only necessary states rather than bulk saving
//!   all callee-saved registers. Inline-able `set_jump` without procedure call cost.
#![cfg_attr(feature = "unstable-asm-goto", feature(asm_goto))]
#![cfg_attr(feature = "unstable-asm-goto", feature(asm_goto_with_outputs))]
#![cfg_attr(not(test), no_std)]
use core::hint::black_box;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::num::NonZero;

#[cfg(target_arch = "x86_64")]
#[macro_use]
#[path = "./x86_64.rs"]
mod imp;

#[cfg(target_arch = "x86")]
#[macro_use]
#[path = "./x86.rs"]
mod imp;

#[cfg(target_arch = "riscv64")]
#[macro_use]
#[path = "./riscv64.rs"]
mod imp;

#[cfg(target_arch = "riscv32")]
#[macro_use]
#[path = "./riscv32.rs"]
mod imp;

#[cfg(target_arch = "aarch64")]
#[macro_use]
#[path = "./aarch64.rs"]
mod imp;

#[cfg(target_arch = "arm")]
#[macro_use]
#[path = "./arm.rs"]
mod imp;

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "riscv64",
    target_arch = "riscv32",
    target_arch = "aarch64",
    target_arch = "arm",
)))]
#[macro_use]
mod imp {
    use super::NonZero;

    compile_error!("sjlj2: unsupported platform");

    pub type Buf = [usize; 0];

    macro_rules! set_jump_raw {
        ($val:tt, $($tt:tt)*) => {
            $val = 0 as _
        };
    }

    pub(crate) unsafe fn long_jump_raw(_buf: *mut (), _result: NonZero<usize>) -> ! {
        unimplemented!()
    }
}

/// A jump checkpoint that you can go back to at any time.
///
/// It consists of a single machine word.
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
    #[must_use]
    pub fn as_raw(self) -> *mut () {
        self.0
    }

    /// Alias of [`set_jump`].
    #[inline(always)]
    pub fn set_jump<T, F, G>(ordinary: F, lander: G) -> T
    where
        F: FnOnce(JumpPoint<'_>) -> T,
        G: FnOnce(NonZero<usize>) -> T,
    {
        set_jump(ordinary, lander)
    }

    /// Alias of [`long_jump`].
    ///
    /// # Safety
    ///
    /// See [`long_jump`].
    #[inline(always)]
    pub unsafe fn long_jump(self, result: NonZero<usize>) -> ! {
        long_jump(self, result)
    }
}

/// Set a jump checkpoint.
///
/// Set a jump checkpoint and execute `ordinary` with a checkpoint argument and return its result.
/// If a long jump is issued on the checkpoint inside execution of `ordinary`, control flow goes
/// back, execute `lander` and return its result from `set_jump`.
///
/// This function is even inline-able without procedure-call cost!
///
/// # Safety
///
/// Yes, this function is actually safe to use. [`long_jump`] is unsafe, however.
#[doc(alias = "setjmp")]
#[inline(always)]
pub fn set_jump<T, F, G>(ordinary: F, lander: G) -> T
where
    F: FnOnce(JumpPoint<'_>) -> T,
    G: FnOnce(NonZero<usize>) -> T,
{
    let mut buf = <MaybeUninit<imp::Buf>>::uninit();
    let ptr = buf.as_mut_ptr();
    let mut val: usize;

    // Show the optimizer that `ordinary` may be called in both ordinary and landing paths.
    // So it cannot assume that variables that are captured by `ordinary` are unchanged in the
    // lander path -- they must be reloaded.
    // This fixes <https://github.com/rust-lang/rfcs/issues/2625>.
    if size_of::<F>() == 0 {
        // F must not mutate local states because it captures nothing.
    } else {
        black_box(&ordinary);
    }

    #[cfg(feature = "unstable-asm-goto")]
    unsafe {
        set_jump_raw!(val, ptr, {
            unsafe { return lander(NonZero::new_unchecked(val)) }
        });
        ordinary(JumpPoint::from_raw(ptr.cast()))
    }

    #[cfg(not(feature = "unstable-asm-goto"))]
    unsafe {
        set_jump_raw!(val, ptr);
        match NonZero::new(val) {
            None => ordinary(JumpPoint::from_raw(ptr.cast())),
            Some(val) => lander(val),
        }
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
