// WAIT: <https://github.com/rust-lang/rfcs/pull/3804>
fn main() {
    println!("cargo::rustc-check-cfg=cfg(emit_cfi)");
    let emit_cfi = std::env::var("CARGO_CFG_WINDOWS").is_err()
        && !matches!(std::env::var("CARGO_CFG_PANIC"), Ok(v) if v == "abort");
    if emit_cfi {
        println!("cargo::rustc-cfg=emit_cfi");
    }
}
