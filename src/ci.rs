//! Defines some utilities that can be used to write the CI pipelines of Rust projects.

use anyhow::{Context, Result, anyhow};
use clap::{Parser, ValueEnum};
use std::{fmt, fs, path::Path, process::Command};

// Re-exported so that clients can use and modify [`Cli`] using a compatible version of [`clap`].
pub use clap;

/// Executes a command and returns an error if it fails.
///
/// # Parameters
/// - `command` is the command that has to be executed.
/// - `description` is the description of the command. It should be a complete sentence, that is it should start with a capital letter and end with a period.
///
/// # Returns
/// Returns [`None`] if and only if it fails.
///
/// # Panics
/// Panics if the command can't be executed.
pub fn execute_command(command: &mut Command, description: &str) -> Option<()> {
    let output = command.output().unwrap_or_else(|_| {
        panic!("Failed to execute the command `{:?}`", command);
    });
    let mut log = String::from(description);
    if !output.stdout.is_empty() {
        log.push('\n');
        log.push_str(&String::from_utf8_lossy(&output.stdout));
    };
    if !output.stderr.is_empty() {
        log.push('\n');
        log.push_str(&String::from_utf8_lossy(&output.stderr));
    };
    tracing::info!("{}", log);
    if output.status.success() {
        Some(())
    } else {
        tracing::error!("Failed");
        None
    }
}

/// Checks the formatting of all the files whose paths are passed in input.
///
/// Fails if there are any formatting errors in any of the examinated files. The type of each file is determined based on its extension.
pub fn format_files<P, Paths>(files: Paths) -> Option<()>
where
    P: AsRef<Path>,
    Paths: IntoIterator<Item = P>,
{
    let _span = tracing::error_span!("Formatting").entered();

    for file_path in files.into_iter() {
        let file_path = file_path.as_ref();
        let file_type = FormatType::parse(file_path)
            .inspect_err(|e| {
                tracing::error!("{:?}", e);
            })
            .ok()?;
        let file_contents = fs::read_to_string(file_path)
            .with_context(|| format!("Couldn't read the contents of '{}'.", file_path.display()))
            .inspect_err(|e| {
                tracing::error!("{:?}", e);
            })
            .ok()?;
        let formatted = file_type
            .format(&file_contents)
            .with_context(|| format!("Couldn't format the contents of '{}'.", file_path.display()))
            .inspect_err(|e| {
                tracing::error!("{:?}", e);
            })
            .ok()?;
        if formatted != file_contents {
            fs::write(file_path, &formatted)
                .with_context(|| {
                    format!(
                        "Couldn't overwrite '{}' with its reformatted version",
                        file_path.display()
                    )
                })
                .inspect_err(|e| {
                    tracing::error!("{:?}", e);
                })
                .ok()?;
            tracing::error!(
                "'{}' wasn't formatted correctly and has been formatted.",
                file_path.display()
            );
            return None;
        }
    }

    tracing::info!("All the files are formatted correctly.");
    Some(())
}

/// Used to specify which file type has to be formatted.
enum FormatType {
    Toml,
    Yaml,
}

impl FormatType {
    /// Tries to initialize a [`FormatType`] instance from the extension of `file_path`.
    ///
    /// Fails if `file_path` doesn't have an extension or it isn't valid.
    pub fn parse<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let file_path = file_path.as_ref();
        let extension = file_path
            .extension()
            .ok_or(anyhow!(
                "'{}' doesn't have a file extension.",
                file_path.display()
            ))?
            .to_str()
            .ok_or(anyhow!(
                "'{}' doesn't have an UTF-8 file extension.",
                file_path.display()
            ))?;
        match extension {
            "toml" => Ok(Self::Toml),
            "yaml" | "yml" => Ok(Self::Yaml),
            _ => Err(anyhow!(
                "The extension of '{}' isn't supported file extension.",
                file_path.display()
            )),
        }
    }

    /// Tries to parse and format `file_contents`.
    ///
    /// # Returns
    /// The formatted version of `file_contents`.
    pub fn format(&self, file_contents: &str) -> anyhow::Result<String> {
        match self {
            Self::Toml => {
                use taplo::formatter::{Options, format};

                let formatting_options = Options {
                    reorder_keys: true,
                    reorder_arrays: true,
                    ..Options::default()
                };
                Ok(format(file_contents, formatting_options))
            }
            Self::Yaml => {
                use pretty_yaml::{config::FormatOptions, format_text};

                format_text(file_contents, &FormatOptions::default())
                    .context("Couldn't parse the contents of the file as valid YAML.")
            }
        }
    }
}

impl fmt::Display for FormatType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Toml => "TOML",
            Self::Yaml => "YAML",
        };
        write!(f, "{}", s)
    }
}

/// Checks for unused dependencies.
///
/// This function is basically a wrapper arount a call to [`cargo-machete`]. If [`cargo-machete`] is not present, it will be installed. The installed version is pinned and is not configurable for simplicity.
///
/// [`cargo-machete`]: https://crates.io/crates/cargo-machete
pub fn check_for_unused_deps() -> Option<()> {
    let _span = tracing::error_span!("Dependencies").entered();
    execute_command(
        Command::new("cargo").args(["install", "cargo-machete@0.9.1", "--locked"]),
        "Installing `cargo-machete`.",
    )?;
    execute_command(
        Command::new("cargo-machete").args(["--with-metadata"]),
        "Checking for unused dependencies.",
    )
}

/// Used to control whether the different packages of the workspace share the same [target directory](https://doc.rust-lang.org/nightly/cargo/reference/build-cache.html) or not.
///
/// [Sharing][Self::Shared] is the standard behaviour and it allows Cargo to cache compilations. This reduces the final size of the target directory and reduces build times, given that Cargo can reuse the compiled artificats for the shared dependencies. The problem with sharing is that some dependencies have to be recompiled when checking the different packages that belong to the workspace, a fact that slows down CI on repeated runs.
///
/// [Sharing][Self::Shared] is therefore ideal for GitHub Actions, were the compilation tends to happen from scratch, while [target directory isolation][Self::Isolated] is better suited for local development if disk usage isn't an issue, as it will significantly increase the iteration speed.
#[derive(Clone, Copy, Default, Debug, ValueEnum)]
pub enum TargetDirType {
    #[default]
    Shared,
    Isolated,
}

impl TargetDirType {
    /// Returns the relative path of the target directory.
    ///
    /// If `self` is [`TargetDirType::Shared`], then the path of the target directory will be `"target/"`, [otherwise][TargetDirType::Isolated] it will be `"target/{subdir_name}"`.
    pub fn get_target_dir_path(&self, subdir_name: &str) -> String {
        match self {
            TargetDirType::Shared => "target".to_string(),
            TargetDirType::Isolated => format!("target/{subdir_name}"),
        }
    }
}

/// Defines the CLI that's commonly used for CI script.
#[derive(Debug, Parser)]
#[command(about = "Runs the CI.")]
pub struct Cli {
    /// Controls whether the target directory will be shared among all the packages of this repository or if each of them will get its own dedicated one.
    ///
    /// See the documentation of [`TargetDirType`] for more information.
    #[arg(short, long, default_value = "shared")]
    pub target_dir_type: TargetDirType,
}
