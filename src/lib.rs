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
//!   all callee-saved registers. Inline-able `set_jump` and `long_jump`.
//!
//!   - 2.4ns `set_jump` setup and 2.9ns `long_jump` on a modern x86\_64 CPU.
//!     ~300-490x faster than `catch_unwind`-`panic_any!`.
//!
//! - `no_std` support.
//!
//!   By default, this crate is `#![no_std]` and does not use `alloc` either.
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
//! No feature is enabled by default.
//!
//! - `unwind`: Enables unwinding across [`set_jump`] boundary from its `ordinary` closure, by
//!   catching and resuming it. This feature requires `std`.
//!
//! ## Supported architectures
//!
//! - x86 (i686)
//! - x86\_64
//! - riscv64
//! - riscv32, with or without E-extension
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
#![cfg_attr(not(any(test, feature = "unwind")), no_std)]
use core::marker::PhantomData;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::num::NonZero;
use core::ops::ControlFlow;

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
#[cfg(any(windows, panic = "abort"))]
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

#[cfg(all(target_arch = "x86", not(target_env = "msvc")))]
#[macro_use]
#[path = "./x86.rs"]
mod imp;

#[cfg(all(target_arch = "x86", target_env = "msvc"))]
#[macro_use]
#[path = "./x86_msvc.rs"]
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
/// # Precondition
///
/// `ordinary` closure must not have a significant `Drop`, or the call frame cannot be POF.
/// We did a best-effort detection for this with [`core::mem::needs_drop`] and a
/// compiler error will be generated for `ordinary` with significant `Drop`.
/// It may (but practically never) generates false positive compile errors.
///
/// # Safety
///
/// Yes, this function is safe to call. [`long_jump`] is unsafe, however.
///
/// # Panics
///
/// It is safe to panic (unwind) in `ordinary` but the behavior varies:
/// - If cargo feature `unwind` is enabled, panic will be caught, passed through
///   `set_jump` boundary and resumed.
/// - Otherwise,  it aborts the process.
///
/// `lander` or `Drop` of `T` can panic, because they are executed
/// outside the `set_jump` boundary.
#[doc(alias = "setjmp")]
#[inline]
pub fn set_jump<T, F, G>(ordinary: F, lander: G) -> T
where
    F: FnOnce(JumpPoint<'_>) -> T,
    G: FnOnce(NonZero<usize>) -> T,
{
    let mut ret = MaybeUninit::uninit();

    #[cfg(feature = "unwind")]
    match set_jump_impl(|jp| {
        ret.write(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || ordinary(jp),
        )));
    }) {
        // SAFETY: `ordinary` returns normally or caught a panic, thus `ret` is initialized.
        ControlFlow::Continue(()) => match unsafe { ret.assume_init() } {
            Ok(ret) => ret,
            Err(payload) => std::panic::resume_unwind(payload),
        },
        ControlFlow::Break(val) => lander(val),
    }

    #[cfg(not(feature = "unwind"))]
    match set_jump_impl(|jp| {
        ret.write(ordinary(jp));
    }) {
        // SAFETY: `ordinary` returns normally, thus `ret` is initialized.
        ControlFlow::Continue(()) => unsafe { ret.assume_init() },
        ControlFlow::Break(val) => lander(val),
    }
}

#[inline]
fn set_jump_impl<F>(ordinary: F) -> ControlFlow<NonZero<usize>>
where
    F: FnOnce(JumpPoint<'_>),
{
    // NB: Properties expected by ASM:
    // - `jmp_buf` is at offset 0.
    // - On the exceptional path, the carried value is stored in `jmp_buf[0]`.
    #[repr(C)]
    struct Data<F> {
        jmp_buf: MaybeUninit<imp::Buf>,
        func: ManuallyDrop<F>,
    }

    macro_rules! gen_wrap {
        ($abi:literal) => {
            unsafe extern $abi fn wrap<F: FnOnce(JumpPoint<'_>)>(data: &mut Data<F>) -> usize {
                // Non-unwinding ABI generates abort-on-unwind guard since our MSRV 1.87.
                // No need to handle unwinding here.
                let jp = unsafe { JumpPoint::from_raw(data.jmp_buf.as_mut_ptr().cast()) };
                unsafe { ManuallyDrop::take(&mut data.func)(jp) };
                0
            }
        };
    }

    // Linux and Windows have different C ABI. Here we choose sysv64 for simplicity.
    #[cfg(target_arch = "x86_64")]
    gen_wrap!("sysv64");

    // x86 cdecl pass all arguments on stack, which is inconvenient under the
    // fact that compilers also disagree on stack alignments.
    // Here we choose fastcall to pass through ECX for simplicity.
    #[cfg(target_arch = "x86")]
    gen_wrap!("fastcall");

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    gen_wrap!("C");

    const {
        assert!(
            !core::mem::needs_drop::<F>(),
            "set_jump closures must not have a significant Drop",
        );
    }

    let mut data = Data::<F> {
        jmp_buf: MaybeUninit::uninit(),
        func: ManuallyDrop::new(ordinary),
    };

    unsafe {
        set_jump_raw!(&raw mut data, wrap::<F>, {
            unsafe {
                let val = NonZero::new_unchecked(data.jmp_buf.assume_init().0[0]);
                return ControlFlow::Break(val);
            }
        });
        ControlFlow::Continue(())
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
