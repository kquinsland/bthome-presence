// Swaps in the correct memory.x file based on the selected chip by way of --feature flag at build time.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn linker_data() -> &'static [u8] {
    #[cfg(feature = "nrf52832")]
    return include_bytes!("memory-nrf52832.x");

    #[cfg(feature = "nrf52810")]
    return include_bytes!("memory-nrf52810.x");

    #[cfg(any(feature = "nrf52832", feature = "nrf52810"))]
    panic!("Unable to build examples for currently selected chip due to missing chip-specific linker configuration (memory.x)");
    // We should never reach this point; only here to satisfy the linter since we're supposed to
    // return something useful and not ().
    &[0]
}

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(linker_data())
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    // Inject git tag as the version number to override the one in Cargo.toml
    // See: https://github.com/rust-lang/cargo/issues/6583#issuecomment-1259871885
    if let Ok(val) = std::env::var("RELEASE_VERSION") {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", val);
    }
    println!("cargo:rerun-if-env-changed=RELEASE_VERSION");
}
