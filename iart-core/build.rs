fn main() {
    let trace_max = std::env::var("WIRT_TRACE_MAX").unwrap_or_else(|_| "1024".to_string());
    println!("cargo:rustc-env=BACK_TRACE_MAX={}", trace_max);
}
