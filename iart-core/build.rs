fn main() {
    let trace_max = std::env::var("IART_TRACE_MAX").unwrap_or_else(|_| "1024".to_string());
    let trace_type = std::env::var("IART_TRACE_TYPE").unwrap_or_else(|_| "good".to_string());
    let track_max = std::env::var("IART_TRACK_MAX").unwrap_or_else(|_| "10".to_string());

    println!("cargo:rustc-env=IART_TRACE_MAX={}", trace_max);
    println!("cargo:rustc-env=IART_TRACE_TYPE={}", trace_type);
    println!("cargo:rustc-env=IART_TRACE_MAX={}", track_max)
}
