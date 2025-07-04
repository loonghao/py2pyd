// build.rs - Conditional mimalloc configuration for cross-compilation
use std::env;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    let host = env::var("HOST").unwrap_or_default();

    println!("cargo:rerun-if-env-changed=TARGET");
    println!("cargo:rerun-if-env-changed=HOST");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ENV");

    // Determine if we should enable mimalloc
    let should_enable_mimalloc = should_use_mimalloc(&target, &host);

    if should_enable_mimalloc {
        println!("cargo:rustc-cfg=feature=\"mimalloc\"");
        println!("cargo:warning=Enabling mimalloc for target: {target}");
    } else {
        println!("cargo:warning=Disabling mimalloc for cross-compilation target: {target}");
        // Set multiple environment variables to disable mimalloc in dependencies
        println!("cargo:rustc-env=CARGO_FEATURE_MIMALLOC=0");
        println!("cargo:rustc-env=MIMALLOC_OVERRIDE=0");
        println!("cargo:rustc-env=MIMALLOC_DISABLE=1");
        // Disable mimalloc compilation entirely for problematic targets
        if target.contains("windows-gnu") || target.contains("mingw") {
            println!("cargo:rustc-env=LIBMIMALLOC_SYS_DISABLE=1");
            println!("cargo:rustc-cfg=feature=\"disable-mimalloc\"");
        }
    }
}

fn should_use_mimalloc(target: &str, host: &str) -> bool {
    // Only enable mimalloc for native builds or safe targets
    if target == host {
        // Native build - safe to use mimalloc
        return true;
    }

    // Check for problematic cross-compilation targets
    let problematic_targets = [
        "i686-pc-windows-gnu",
        "x86_64-pc-windows-gnu",
        "aarch64-unknown-linux-musl",
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-gnu",
        // Add more Windows GNU variants to be safe
        "i686-w64-mingw32",
        "x86_64-w64-mingw32",
    ];

    if problematic_targets.iter().any(|&t| target.contains(t)) {
        return false;
    }

    // For other cross-compilation targets, be conservative
    false
}
