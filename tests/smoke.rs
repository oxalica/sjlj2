use std::num::NonZero;

use sjlj2::{long_jump, set_jump};

#[test]
fn smoke() {
    let ret = set_jump(|_| 42, |_| panic!("no jump"));
    assert_eq!(ret, 42);

    let capture = 42;
    let ret = set_jump(
        move |_| capture + 1,
        move |_| {
            let _capture = capture;
            panic!("no jump");
        },
    );
    assert_eq!(ret, 43);

    let ret = set_jump(
        |jp| unsafe { long_jump(jp, NonZero::new(13).unwrap()) },
        |ret| ret.get(),
    );
    assert_eq!(ret, 13);
    let ret = set_jump(
        move |jp| unsafe { long_jump(jp, NonZero::new(capture).unwrap()) },
        move |ret| ret.get() * 100 + capture,
    );
    assert_eq!(ret, 4242);
}
