//! `nexora-rhi-probe`: open the native backend on this machine and prove it.
//!
//! Run by `scripts/local-validation.py` as the `rhi_native` check. Prints the
//! adapter as a device class, runs the conformance suite with this backend's
//! shaders, then uploads, draws and reads back (`nexora_rhi_wgpu::proof`).
//! Exits non-zero at the first failure, with the reason.

use std::process::ExitCode;

use nexora_rhi::conformance::{self, CASES};
use nexora_rhi_wgpu::{conformance_shaders, proof, WgpuRhi};

fn main() -> ExitCode {
    match probe() {
        Ok(()) => {
            println!("result             OK");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("result             FAILED");
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn probe() -> nexora_foundation::error::Result<()> {
    let mut rhi = WgpuRhi::new()?;
    let adapter = rhi.adapter().clone();
    println!(
        "adapter            {} ({}, {})",
        adapter.name, adapter.backend, adapter.kind
    );
    println!("driver             {}", adapter.driver);
    let report = conformance::run(&mut rhi, &conformance_shaders())?;
    println!(
        "conformance        {}/{} cases on {}",
        report.passed.len(),
        CASES.len(),
        report.backend
    );
    let proof = proof::run(&mut rhi)?;
    println!(
        "upload             {} bytes to a 16x16 texture, read back identical",
        proof.uploaded_bytes
    );
    println!(
        "draw               {} of {} texels shaded by the GPU",
        proof.shaded_texels, proof.target_texels
    );
    Ok(())
}
