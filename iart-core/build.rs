fn main() {
    let trace_max = std::env::var("IART_TRACE_MAX").unwrap_or_else(|_| "1024".to_string());
    let trace_type = std::env::var("IART_TRACE_TYPE").unwrap_or_else(|_| "good".to_string());

    println!("cargo:rustc-env=IART_TRACE_MAX={}", trace_max);
    println!("cargo:rustc-env=IART_TRACE_TYPE={}", trace_type);
}
