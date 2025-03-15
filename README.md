# Safer[^1], cheaper and more ergonomic setjmp/longjmp in Rust[^2]

[^1]: `long_jump` is still unsafe and is technically UB, though.
      See more about safety in docs of `long_jump`.

[^2]: ...and assembly. No C trampoline is involved!

[pof]: https://rust-lang.github.io/rfcs/2945-c-unwind-abi.html#plain-old-frames

[![crates.io](https://img.shields.io/crates/v/sjlj2)](https://crates.io/crates/sjlj2)
[![docs.rs](https://img.shields.io/docsrs/sjlj2)][docs]
[![CI](https://github.com/oxalica/async-ffi/actions/workflows/ci.yaml/badge.svg)](https://github.com/oxalica/async-ffi/actions/workflows/ci.yaml)

See more in [documentations][docs].

[docs]: https://docs.rs/sjlj2
