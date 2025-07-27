use std::hint::black_box;
use std::ops::ControlFlow;
use std::panic::{catch_unwind, resume_unwind};

use criterion::{criterion_group, criterion_main, Criterion};
use sjlj2::catch_long_jump;

const NEST_LVL: usize = 20;

#[inline(never)]
fn nest(n: usize, f: &impl Fn()) {
    if n == 0 {
        f();
    } else {
        let _keep = black_box(&n);
        nest(n - 1, f);
    }
}

fn bench_sjlj(c: &mut Criterion) {
    c.bench_function(&format!("nest{NEST_LVL}"), |b| {
        let capture = 42;
        b.iter(|| {
            nest(NEST_LVL, &|| {
                let _ = black_box(capture);
            });
        });
    });

    for (name, jump, lvl, expect) in [
        ("ordinary", false, 0, 42),
        ("jump0", true, 0, 14),
        (&format!("jump{NEST_LVL}"), true, NEST_LVL, 14),
    ] {
        c.bench_function(&format!("sjlj/{name}"), |b| {
            let jump = black_box(jump);
            let lvl = black_box(lvl);
            b.iter(|| {
                let ret = match catch_long_jump(move |jp| {
                    if jump {
                        nest(lvl, &|| unsafe { jp.long_jump(13) });
                    }
                    42usize
                }) {
                    ControlFlow::Continue(ret) => ret,
                    ControlFlow::Break(data) => data + 1,
                };
                assert_eq!(ret, expect);
            });
        });

        c.bench_function(&format!("panic/{name}"), |b| {
            let jump = black_box(jump);
            let lvl = black_box(lvl);
            b.iter(|| {
                let ret = catch_unwind(move || {
                    if jump {
                        nest(lvl, &|| resume_unwind(Box::new(13usize)));
                    }
                    42usize
                });
                let ret = match ret {
                    Ok(x) => x,
                    Err(err) => *err.downcast::<usize>().unwrap() + 1,
                };
                assert_eq!(ret, expect);
            });
        });
    }
}

criterion_group!(bench_group, bench_sjlj);
criterion_main!(bench_group);
