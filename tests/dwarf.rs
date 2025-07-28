//! Test DWARF information correctness by random sampling and unwinding.
#![cfg(target_os = "linux")]
use std::hint::black_box;
use std::ops::ControlFlow;
use std::time::{Duration, Instant};

use sjlj2::catch_long_jump;

const TEST_DURATION: Duration = Duration::from_secs(60);
const CHUNK: usize = 1_000_000;

#[test]
#[ignore = "slow"]
fn dwarf_unwind() {
    let _guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1_000)
        .build()
        .unwrap();
    let inst = Instant::now();
    while inst.elapsed() < TEST_DURATION {
        for _ in 0..CHUNK {
            let ret = catch_long_jump(|jp| {
                if black_box(true) {
                    unsafe { jp.long_jump(13) };
                } else {
                    42
                }
            });
            assert_eq!(ret, ControlFlow::Break(13));
        }
    }
}
