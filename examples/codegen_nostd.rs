//! For manual inspection.
// Workaround: examples are build with std when running `cargo test`. Ignore no_std in that case.
#![cfg_attr(not(feature = "default"), no_std)]

use core::num::NonZero;

use sjlj2::JumpPoint;

#[cfg(not(feature = "default"))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn codegen_call(f: extern "C" fn(*mut ()), g: extern "C" fn(usize)) {
    JumpPoint::set_jump(
        |jp| {
            f(jp.as_raw());
            unsafe { jp.long_jump(NonZero::new(42).unwrap()) };
        },
        |v| {
            g(v.get());
        },
    );
}

#[no_mangle]
extern "C" fn codegen_no_jump() -> usize {
    JumpPoint::set_jump(|_jp| 42, |_| 13)
}

#[no_mangle]
extern "C" fn codegen_must_jump() -> usize {
    JumpPoint::set_jump(
        |jp| unsafe { jp.long_jump(NonZero::new_unchecked(13)) },
        |v| v.get() + 1,
    )
}
