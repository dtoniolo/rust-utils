//! Implements the script that is used to check the code during the continuos integration.
//!
//! For more information see the *Continuous Integration* section of the `README`.

use rust_utils::{
    ci::{check_for_unused_deps, execute_command, format_files},
    logging::init_logger,
};

use std::process::Command;

fn main() -> Result<(), ()> {
    init_logger();
    run_ci_pipeline().ok_or(())
}

/// Runs the commands that compose the CI pipeline.
///
/// # Returns
/// Returns [`None`] if and only if it fails.
fn run_ci_pipeline() -> Option<()> {
    run_ci_pipeline_rust_utils()?;
    check_for_unused_deps()?;
    format_files([
        ".cargo/config.toml",
        "Cargo.toml",
        "rust-toolchain.toml",
        ".github/workflows/ci.yml",
        ".github/dependabot.yml",
    ])
}

/// Runs the commands that compose the CI pipeline for the [`rust-utils`][rust_utils] package.
///
/// # Returns
/// Returns [`None`] if and only if it fails.
fn run_ci_pipeline_rust_utils() -> Option<()> {
    let _pkg_span = tracing::error_span!("Check `rust-utils`").entered();
    let span = tracing::error_span!("Clippy").entered();
    execute_command(
        Command::new("cargo").args(["clippy", "--package", "rust-utils"]),
        "Checking the source code",
    )?;
    span.exit();

    let span = tracing::error_span!("Docs").entered();
    execute_command(
        Command::new("cargo").args(["doc", "--package", "rust-utils"]),
        "Generating the public documentation.",
    )?;
    execute_command(
        Command::new("cargo").args(["doc", "--package", "rust-utils", "--document-private-items"]),
        "Generating the private documentation.",
    )?;
    span.exit();

    let _span = tracing::error_span!("Format").entered();
    execute_command(
        Command::new("cargo").args(["fmt", "--check", "--package", "rust-utils"]),
        "Checking the formatting of the code.",
    )
}
