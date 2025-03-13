//! For manual inspection.
use std::num::NonZero;

use sjlj2::{long_jump, set_jump};

#[no_mangle]
extern "C" fn codegen_call(f: extern "C" fn(*mut ()), g: extern "C" fn(usize)) {
    set_jump(
        |jp| {
            f(jp.as_raw());
            unsafe { long_jump(jp, NonZero::new(42).unwrap()) };
        },
        |v| {
            g(v.get());
        },
    );
}

#[no_mangle]
extern "C" fn codegen_no_jump() -> usize {
    set_jump(|_jp| 42, |_| 13)
}

#[no_mangle]
extern "C" fn codegen_must_jump() -> usize {
    set_jump(
        |jp| unsafe { long_jump(jp, NonZero::new_unchecked(13)) },
        |v| v.get() + 1,
    )
}

fn main() {}
