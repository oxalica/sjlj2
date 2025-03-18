//! # Safer[^1], cheaper and more ergonomic setjmp/longjmp in Rust[^2].
//!
//! [^1]: [`long_jump`] is still unsafe and is technically UB, though.
//!       See more about safety in [`long_jump`].
//! [^2]: ...and assembly. No C trampoline is involved!
//!
//! - Ergonomic and safer* Rusty API for typical usages. Closure API instead of multiple-return.
//!
//!   Multiple-return functions are undefined behaviors due to
//!   [fatal interaction with optimizer][misopt].
//!   This crate does not suffer from the misoptimization (covered in `tests/smoke.rs`).
//!
//!   ⚠️  We admit that since it's not yet undefined to force unwind Rust POFs,
//!   `long_jump` is still technically undefined behavior.
//!   But this crate is an attempt to make a semantically-correct abstraction free from
//!   misoptimization, and you accept the risk by using this crate.
//!   If you find any misoptimization in practice, please open an issue.
//!
//! - Single-use jump checkpoint.
//!
//!   No jump-after-jump disaster. No coroutine-at-home.
//!
//! - Minimal memory and performance footprint.
//!
//!   Single `usize` `JumpPoint`. Let optimizer save only necessary states rather than bulk saving
//!   all callee-saved registers. Inline-able `set_jump` without procedure call cost.
//!
//!   - 2.4ns `set_jump` setup and 2.9ns `long_jump` on a modern x86\_64 CPU.
//!     ~300-490x faster than `catch_unwind`-`panic_any!`.
//!
//! - No std.
//!
//!   This crate is `#[no_std]` and does not use `alloc` either.
//!   It is suitable for embedded environment.
//!
//! ```
//! use std::num::NonZero;
//! use sjlj2::JumpPoint;
//!
//! let mut a = 42;
//! // Execute with a jump checkpoint. Both closures can return a value.
//! let b = JumpPoint::set_jump(
//!     // The ordinary path that is always executed.
//!     |jump_point| {
//!         a = 13;
//!         // Jump back to the alternative path with a `NonZero<usize>` value.
//!         // SAFETY: All frames between `set_jump` and `long_jump` are POFs.
//!         unsafe {
//!             jump_point.long_jump(NonZero::new(99).unwrap());
//!         }
//!     },
//!     // The alternative path which is only executed once `long_jump` is called.
//!     |v| {
//!         // The carried value.
//!         v.get()
//!     }
//! );
//! assert_eq!(a, 13);
//! assert_eq!(b, 99);
//! ```
//!
//! ## Features
//!
//! - `unstable-asm-goto`: enable use of `asm_goto` and `asm_goto_with_outputs` unstable features.
//!
//!   This requires a nightly rustc, but produces more optimal code with one-less conditional jump.
//!
//!   ⚠️ Warning: `asm_goto_with_outputs` is [reported to be buggy][asm_goto_bug] in some cases. It
//!   is unknown that if our code is affected. Do NOT enable this feature unless you accept the
//!   risk.
//!
//! [asm_goto_bug]: https://github.com/llvm/llvm-project/issues/74483
//!
//! ## Supported architecture
//!
//! - x86 (i686)
//! - x86\_64
//! - riscv64
//! - riscv32 (with and without E-extension)
//! - aarch64 (ARMv8)
//! - arm
//!
//! ## Similar crates
//!
//! - [`setjmp`](https://crates.io/crates/setjmp)
//!
//!   - Generates from C thus needs a correctly-setup C compiler to build.
//!   - Unknown performance because it fails to build for me. (Poor compatibility?)
//!   - Suffers from [misoptimization][misopt].
//!
//! - [`sjlj`](https://crates.io/crates/sjlj)
//!
//!   - Only x86\_64 is supported.
//!   - Suffers from [misoptimization][misopt] due to multi-return.
//!   - Slower `long_jump` because of more register restoring.
//!
//! [misopt]: https://github.com/rust-lang/rfcs/issues/2625
#![cfg_attr(feature = "unstable-asm-goto", feature(asm_goto))]
#![cfg_attr(feature = "unstable-asm-goto", feature(asm_goto_with_outputs))]
#![cfg_attr(not(test), no_std)]
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::num::NonZero;

// Overridable by the next definition, and can be unused on some targets.
#[allow(unused_macros)]
macro_rules! maybe_strip_cfi {
    (($($head:tt)*), $($lit1:literal,)* $([$cfi:literal], $($lit2:literal,)*)* [], $($tail:tt)*) => {
        $($head)* (
            $($lit1,)*
            $($cfi, $($lit2,)*)*
            $($tail)*
        )
    };
}

// Windows do not use DWARF unwind info.
#[cfg(windows)]
macro_rules! maybe_strip_cfi {
    (($($head:tt)*), $($lit1:literal,)* $([$cfi:literal], $($lit2:literal,)*)* [], $($tail:tt)*) => {
        $($head)* (
            $($lit1,)*
            $($($lit2,)*)*
            $($tail)*
        )
    };
}

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
pub struct JumpPoint<'a>(*mut (), PhantomData<fn(&'a ()) -> &'a ()>);

#[cfg(doctest)]
/// ```compile_fail
/// fn f(j: sjlj2::JumpPoint<'_>) -> impl Send { j }
/// ```
///
/// ```compile_fail
/// fn f(j: sjlj2::JumpPoint<'_>) -> impl Sync { j }
/// ```
fn _assert_not_or_sync() {}

#[cfg(doctest)]
/// ```compile_fail
/// fn f<'a, 'b: 'a>(j: sjlj2::JumpPoint<'a>) -> sjlj2::JumpPoint<'b> { j }
/// ```
///
/// ```compile_fail
/// fn f<'a: 'b, 'b>(j: sjlj2::JumpPoint<'a>) -> sjlj2::JumpPoint<'b> { j }
/// ```
fn _assert_invariant() {}

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
    #[inline]
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
    #[inline]
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
/// `ordinary` and `lander` must not have significant `Drop`, otherwise a compile error is
/// generated. The call frame cannot be POF if any of them has a significant `Drop` impl.
/// Since it is implemented with [`core::mem::needs_drop`], it may generates false positive
/// compile errors.
///
/// This function is inline-able but `ordinary` is called in a C function wrapper because it must
/// has its own stack frame to avoid [misoptimization][misopt2].
///
/// [misopt2]: https://github.com/rust-lang/libc/issues/1596
///
/// # Safety
///
/// Yes, this function is actually safe to use. [`long_jump`] is unsafe, however.
///
/// # Panics
///
/// It is allowed to panic in `ordinary` or `lander` closure. But panics in `ordinary` cannot
/// unwind across `set_jump`, or it will abort the process.
/// Panics in `lander` can unwind across `set_jump` though.
#[doc(alias = "setjmp")]
#[inline]
// For better logical order.
#[allow(clippy::items_after_statements)]
pub fn set_jump<T, F, G>(ordinary: F, lander: G) -> T
where
    F: FnOnce(JumpPoint<'_>) -> T,
    G: FnOnce(NonZero<usize>) -> T,
{
    const {
        assert!(
            !core::mem::needs_drop::<F>(),
            "set_jump closures must not have significant Drop",
        );
        assert!(
            !core::mem::needs_drop::<G>(),
            "set_jump closures must not have significant Drop",
        );
        // `T` can have `Drop` impl because when it is returned, there are no more user code can
        // unwind between its return from F or G and the return from `set_jump`.
    }

    union Data<T, F> {
        func: ManuallyDrop<F>,
        ret: ManuallyDrop<T>,
    }

    struct AbortGuard;
    impl Drop for AbortGuard {
        fn drop(&mut self) {
            // Manually trigger a double-panic abort.
            // Workaround: <https://github.com/rust-lang/rust/issues/67952>
            panic!("panic cannot unwind across set_jump");
        }
    }

    macro_rules! gen_wrap {
        ($abi:literal) => {
            unsafe extern $abi fn wrap<T, F: FnOnce(JumpPoint<'_>) -> T>(
                data: &mut Data<T, F>,
                jp: *mut (),
            ) -> usize {
                let guard = AbortGuard;
                let f = unsafe { ManuallyDrop::take(&mut data.func) };
                let ret = f(unsafe { JumpPoint::from_raw(jp) });
                data.ret = ManuallyDrop::new(ret);
                core::mem::forget(guard);
                0
            }
        };
    }

    #[cfg(target_arch = "x86_64")]
    gen_wrap!("sysv64");
    #[cfg(not(target_arch = "x86_64"))]
    gen_wrap!("C");

    let mut data = Data::<T, F> {
        func: ManuallyDrop::new(ordinary),
    };
    let val: usize;

    #[cfg(feature = "unstable-asm-goto")]
    unsafe {
        set_jump_raw!(val, wrap::<T, F>, &mut data, {
            unsafe { return lander(NonZero::new_unchecked(val)) }
        });
        ManuallyDrop::take(&mut data.ret)
    }

    #[cfg(not(feature = "unstable-asm-goto"))]
    {
        unsafe { set_jump_raw!(val, wrap::<T, F>, &mut data) };
        match NonZero::new(val) {
            None => unsafe { ManuallyDrop::take(&mut data.ret) },
            Some(v) => lander(v),
        }
    }
}

/// Long jump to a checkpoint, and executing corresponding `set_jump`'s `exceptional` closure.
///
/// # Safety
///
/// - When [`long_jump`] is called on the [`JumpPoint`] argument of `ordinary` closure during its
///   execution, all stack frames between that `long_jump` and this `set_jump`, must be all
///   [Plain Old Frames][pof].
///
///   Note: It is explicitly said in [RFC2945][pof] that
///   > When deallocating Rust POFs: for now, this is not specified, and must be considered
///   > undefined behavior.
///
///   But in practice, this crate does not suffers the
///   [relevant optimization issue caused by return-twice][misopt],
///   and should be UB-free as long as only POFs are `long_jump`ed over.
///
/// [pof]: https://rust-lang.github.io/rfcs/2945-c-unwind-abi.html#plain-old-frames
/// [misopt]: https://github.com/rust-lang/rfcs/issues/2625
#[doc(alias = "longjmp")]
#[inline]
pub unsafe fn long_jump(point: JumpPoint<'_>, result: NonZero<usize>) -> ! {
    unsafe { imp::long_jump_raw(point.0, result) }
}
