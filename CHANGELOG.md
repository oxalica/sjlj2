# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/)
and this project adheres to [Semantic Versioning](https://semver.org/).

## v0.4.0

### Changed

- MSRV is bumped to 1.87, and edition 2024 is used.

- The `set_jump` interface is refactored to mimic the `catch_unwind` interface
  and also enables the path elimination optimization.

  The new signature is:
  ```rust
  pub fn catch_long_jump<T, F>(f: F) -> ControlFlow<usize, T>
  where F: FnOnce(JumpPoint<'_>) -> T
  ```

- Explicitly document that `catch_long_jump` is POF, thus 
  `long_jump` across multiple levels of `catch_long_jump` is allowed.

- The type of carried data of `long_jump` is changed from `NonZero<usize>` to `usize`.
  Zero value is now correctly handled, unlike the C/POSIX counterpart.

- `JumpPoint::as_raw` is not a `const fn`.

### Removed

- `unstable-asm-goto` feature. Since `asm_goto` is stabilized under the new MSRV.

### Fixed

- Crash on i686-pc-windows-msvc.

- Crash on aarch64 under nightly rustc.

## v0.3.1

### Fixed

- Fixed build failure with debug=false and panic=abort, by omitting CFI
  directives on panic=abort (#1).

## v0.3.0

### Changed

- Refactored multi-return `asm!` into single-return with wrapper calls.

  This forbids panicking from `ordinary` closure by default since unwinding
  from asm blocks is unstable yet.

  See: <https://doc.rust-lang.org/stable/reference/inline-assembly.html#r-asm.rules.only-on-exit>

  See also: <https://github.com/rust-lang/rfcs/issues/2625#issuecomment-2727671210>

### Added

- Added a default-disabled `unwind` feature to enable unwinding across
  `set_jump` boundary. It requires `std`.

### Fixed

- Fixed misoptimization under aarch64-apple-darwin with asm-goto.

- Fixed misoptimization caused by incorrect stack slots reuse.

## v0.2.0

### Changed

- Replace `black_box` hint with a stronger `asm` black box.

- Reject significant Drop impls on `set_jump` arguments in best efforts.

### Fixed

- Fixed compilation on aarch64-apple-darwin.

- Fixed typos an update documentations.

## v0.1.0

Initial release.
