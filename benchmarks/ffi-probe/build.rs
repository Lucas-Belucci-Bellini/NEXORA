//! Compiles and links the C++ side of the boundary, when asked to.
//!
//! Hand-rolled rather than using the `cc` crate, because ADR-0002 keeps this
//! workspace at zero external dependencies and a build script is still part of
//! the build. Two process invocations and three `cargo:` directives is the whole
//! job; a dependency to save that is a dependency that has to be audited,
//! vendored and kept current for the life of the project.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The translation unit that lives on the far side of the boundary.
///
/// Shared with `benchmarks/cpp`, deliberately: measuring a boundary against a
/// *copy* of the far side would measure a boundary that does not exist.
const BOUNDARY_SOURCE: &str = "../cpp/src/ffi_boundary.cpp";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={BOUNDARY_SOURCE}");
    println!("cargo:rerun-if-env-changed=CXX");
    println!("cargo:rerun-if-env-changed=AR");
    // Declared unconditionally: with the feature off the cfg is simply never
    // set, but `rustc` still has to be told the name is expected. CI builds
    // with `-D warnings`, so an unexpected-cfg lint is a failed build.
    println!("cargo:rustc-check-cfg=cfg(cpp_boundary_linked)");

    if env::var_os("CARGO_FEATURE_CPP").is_none() {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("cargo always sets OUT_DIR"));
    let source = Path::new(BOUNDARY_SOURCE);
    assert!(
        source.exists(),
        "the `cpp` feature is on but {BOUNDARY_SOURCE} is missing; \
         the C++ reference is part of this repository, not an optional download"
    );

    let object = out_dir.join("ffi_boundary.o");
    let archive = out_dir.join("libnexora_ffi_boundary.a");

    // -fno-lto is the point of the exercise: an inlined boundary is not a
    // boundary. -O2 matches how the C++ reference is built in benchmarks/cpp,
    // so the far side is not handicapped relative to the near side.
    let compiler = env::var("CXX").unwrap_or_else(|_| "c++".to_string());
    run(
        Command::new(&compiler)
            .args([
                "-std=c++20",
                "-O2",
                "-fno-lto",
                "-fPIC",
                "-c",
                BOUNDARY_SOURCE,
                "-o",
            ])
            .arg(&object),
        &compiler,
    );

    let archiver = env::var("AR").unwrap_or_else(|_| "ar".to_string());
    run(
        Command::new(&archiver)
            .arg("crs")
            .arg(&archive)
            .arg(&object),
        &archiver,
    );

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=nexora_ffi_boundary");
    // Read by `src/lib.rs` to decide whether the extern block exists at all.
    println!("cargo:rustc-cfg=cpp_boundary_linked");
}

/// Run a build step, failing loudly.
///
/// Startup brief §45: a build that silently produced no boundary would leave
/// the benchmark reporting Rust-to-Rust numbers under a cross-language label.
fn run(command: &mut Command, program: &str) {
    let status = command.status().unwrap_or_else(|error| {
        panic!(
            "could not run `{program}`: {error}. The `cpp` feature needs a C++ \
             compiler and `ar` on PATH; build without it for a pure-Rust workspace."
        )
    });
    assert!(status.success(), "`{program}` failed: {status}");
}
