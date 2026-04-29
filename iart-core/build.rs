fn main() {
    println!("cargo:rerun-if-env-changed=IART_TRACE_MAX");
    println!("cargo:rerun-if-env-changed=IART_TRACK_MAX");

    let trace_max = std::env::var("IART_TRACE_MAX").unwrap_or_else(|_| "0".to_string());
    let trace_type = std::env::var("IART_TRACE_TYPE").unwrap_or_else(|_| "good".to_string());
    let track_max = std::env::var("IART_TRACK_MAX").unwrap_or_else(|_| "16".to_string());
    let track_offset_max =
        std::env::var("IART_TRACKER_MAX_OFFSET").unwrap_or_else(|_| "8".to_string());

    println!("cargo:rustc-env=IART_TRACE_MAX={}", trace_max);
    println!("cargo:rustc-env=IART_TRACE_TYPE={}", trace_type);
    println!("cargo:rustc-env=IART_TRACK_MAX={}", track_max);
    println!(
        "cargo:rustc-env=IART_TRACKER_MAX_OFFSET={}",
        track_offset_max
    );
}
