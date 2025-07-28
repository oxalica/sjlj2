//! For manual inspection.
// Workaround: examples are build with std when running `cargo test`. Ignore no_std in that case.
#![cfg_attr(not(feature = "default"), no_std)]

use core::{convert::Infallible, ops::ControlFlow};

use sjlj2::catch_long_jump;

#[cfg(not(feature = "default"))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
extern "C" fn codegen_call(f: extern "C" fn(*mut ()), g: extern "C" fn(usize)) {
    match catch_long_jump(|jp| {
        f(jp.as_raw());
        unsafe { jp.long_jump(42) };
    }) {
        ControlFlow::Continue(()) => {}
        ControlFlow::Break(data) => g(data),
    }
}

#[unsafe(no_mangle)]
extern "C" fn codegen_no_jump() -> bool {
    catch_long_jump(|_jp| 42).is_break()
}

#[unsafe(no_mangle)]
extern "C" fn codegen_must_jump() -> bool {
    catch_long_jump::<Infallible, _>(|jp| unsafe { jp.long_jump(13) }).is_break()
}
