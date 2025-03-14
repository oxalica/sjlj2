use std::num::NonZero;
use std::panic::{catch_unwind, panic_any};

use sjlj2::JumpPoint;

#[test]
fn smoke() {
    let ret = JumpPoint::set_jump(|_| 42, |_| panic!("no jump"));
    assert_eq!(ret, 42);

    let capture = 42;
    let ret = JumpPoint::set_jump(
        move |_| capture + 1,
        move |_| {
            // Force capture.
            #[allow(clippy::no_effect_underscore_binding)]
            let _capture = capture;
            panic!("no jump");
        },
    );
    assert_eq!(ret, 43);

    let ret = JumpPoint::set_jump(
        |jp| unsafe { jp.long_jump(NonZero::new(13).unwrap()) },
        |v| v.get() + 1,
    );
    assert_eq!(ret, 14);
    let ret = JumpPoint::set_jump(
        move |jp| unsafe { jp.long_jump(NonZero::new(capture).unwrap()) },
        move |ret| ret.get() * 100 + capture,
    );
    assert_eq!(ret, 4242);
}

#[test]
fn issue2625() {
    #[inline(never)]
    fn foo() -> (usize, usize) {
        let mut x = 42usize;
        let y = JumpPoint::set_jump(
            |jp| {
                // Step 0: setjmp returns 0
                // Step 1: x is modified
                x = 13;
                // Step 2: jumps to Step 0 returning 1
                unsafe { jp.long_jump(NonZero::new(1).unwrap()) };
            },
            |y| {
                // Step 3: when setjmp returns 1 x has always been
                // modified to be  == 13 so this should always return 13:
                y.get() + 1
            },
        );
        // The optimizer must not assume `x` is unchanged in the long_jump lander path.
        (x, y)
    }

    assert_eq!(foo(), (13, 2));
}

#[test]
fn panic() {
    let ret = catch_unwind(|| {
        JumpPoint::set_jump(|_| panic_any(42usize), |_| {});
    });
    let payload = *ret.unwrap_err().downcast::<usize>().unwrap();
    assert_eq!(payload, 42usize);

    let ret = catch_unwind(|| {
        JumpPoint::set_jump(
            |jp| unsafe { jp.long_jump(NonZero::new(13).unwrap()) },
            |v| panic_any(v.get()),
        );
    });
    let payload = *ret.unwrap_err().downcast::<usize>().unwrap();
    assert_eq!(payload, 13usize);
}
