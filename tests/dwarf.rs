//! Test DWARF information correctness by random sampling and unwinding.
#![cfg(target_os = "linux")]
use std::num::NonZero;
use std::time::{Duration, Instant};

use sjlj2::{long_jump, set_jump};

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
            let ret = set_jump(
                |jp| unsafe { long_jump(jp, NonZero::new(13).unwrap()) },
                |ret| ret.get() + 1,
            );
            assert_eq!(ret, 14);
        }
    }
}
