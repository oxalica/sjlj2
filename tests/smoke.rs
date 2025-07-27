use std::hint::black_box;
use std::num::NonZero;
use std::ops::ControlFlow;
use std::panic::{catch_unwind, panic_any};
use std::ptr::read_volatile;

use sjlj2::catch_long_jump;

#[test]
fn smoke() {
    let ret = catch_long_jump(|_| 42i32);
    assert_eq!(ret, ControlFlow::Continue(42));

    let capture = 42;
    let ret = catch_long_jump(move |_| capture + 1);
    assert_eq!(ret, ControlFlow::Continue(43));

    let ret = catch_long_jump(|jp| unsafe { jp.long_jump(NonZero::new(13).unwrap()) });
    assert_eq!(ret, ControlFlow::Break(NonZero::new(13).unwrap()));
}

#[test]
fn issue_2625() {
    #[inline(never)]
    fn foo() -> (usize, usize) {
        let mut x = 42usize;
        let ret = catch_long_jump(|jp| {
            // Step 0: setjmp returns 0
            // Step 1: x is modified
            x = 13;
            // Step 2: jumps to Step 0 returning 1
            unsafe { jp.long_jump(NonZero::new(1).unwrap()) };
        });
        match ret {
            ControlFlow::Continue(()) => unreachable!(),
            // Step 3: when setjmp returns 1, x has always been
            // modified to be == 13 so this should always return 13:
            ControlFlow::Break(y) => {
                let y = y.get() + 1;
                // The optimizer must not assume `x` is unchanged in the long_jump lander path.
                (x, y)
            }
        }
    }

    assert_eq!(foo(), (13, 2));
}

#[cfg(feature = "unwind")]
#[test]
fn resume_panic() {
    let ret = catch_unwind(|| {
        let _ = catch_long_jump(|_jp| panic_any(42usize));
    });
    let payload = *ret.unwrap_err().downcast::<usize>().unwrap();
    assert_eq!(payload, 42usize);
}

// Test DWARF state or SEH chain restoration.
#[test]
fn after_panic() {
    let ret = catch_unwind(|| {
        let _ = catch_long_jump(|jp| unsafe { jp.long_jump(NonZero::new(1).unwrap()) });
        panic_any(13usize);
    });
    let payload = *ret.unwrap_err().downcast::<usize>().unwrap();
    assert_eq!(payload, 13usize);
}

// <https://github.com/rust-lang/libc/issues/1596>
#[test]
fn libc_issue_1596() {
    pub unsafe fn run() -> [i32; 20] {
        let a0 = read_volatile(&0);
        let a1 = read_volatile(&1);
        let a2 = read_volatile(&2);
        let a3 = read_volatile(&3);
        let a4 = read_volatile(&4);
        let a5 = read_volatile(&5);
        let a6 = read_volatile(&6);
        let a7 = read_volatile(&7);
        let a8 = read_volatile(&8);
        let a9 = read_volatile(&9);
        let a10 = read_volatile(&10);
        let a11 = read_volatile(&11);
        let a12 = read_volatile(&12);
        let a13 = read_volatile(&13);
        let a14 = read_volatile(&14);
        let a15 = read_volatile(&15);
        let a16 = read_volatile(&16);
        let a17 = read_volatile(&17);
        let a18 = read_volatile(&18);
        let a19 = read_volatile(&19);
        let _ = catch_long_jump(|jp| {
            let b0 = read_volatile(&20);
            let b1 = read_volatile(&21);
            let b2 = read_volatile(&22);
            let b3 = read_volatile(&23);
            let b4 = read_volatile(&24);
            let b5 = read_volatile(&25);
            let b6 = read_volatile(&26);
            let b7 = read_volatile(&27);
            let b8 = read_volatile(&28);
            let b9 = read_volatile(&29);
            let b10 = read_volatile(&30);
            let b11 = read_volatile(&31);
            let b12 = read_volatile(&32);
            let b13 = read_volatile(&33);
            let b14 = read_volatile(&34);
            let b15 = read_volatile(&35);
            let b16 = read_volatile(&36);
            let b17 = read_volatile(&37);
            let b18 = read_volatile(&38);
            let b19 = read_volatile(&39);
            black_box(b0);
            black_box(b1);
            black_box(b2);
            black_box(b3);
            black_box(b4);
            black_box(b5);
            black_box(b6);
            black_box(b7);
            black_box(b8);
            black_box(b9);
            black_box(b10);
            black_box(b11);
            black_box(b12);
            black_box(b13);
            black_box(b14);
            black_box(b15);
            black_box(b16);
            black_box(b17);
            black_box(b18);
            black_box(b19);
            unsafe { jp.long_jump(NonZero::new(1).unwrap()) }
        });
        [
            a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18,
            a19,
        ]
    }

    let ret = unsafe { run() };
    assert_eq!(&ret[..], (0..20).collect::<Vec<i32>>());
}
