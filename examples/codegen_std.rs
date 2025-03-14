//! For manual inspection.
use std::num::NonZero;

use sjlj2::JumpPoint;

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

fn main() {}
