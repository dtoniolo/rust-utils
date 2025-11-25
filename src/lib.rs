//! This crate contains various utilities for Rust projects.

pub mod ci;
pub mod logging;

/// Returns the path of the parent directory of the file in which it is called with the package name removed.
///
/// This is useful when running tests in a package that's part of a workspace: in this case the current working directory is set to the root of the pacakge, but the [`file`] macro returns a path that starts at the root of the repository, resulting in the repetition of the directory in which the package lives. Building a file path relative to the test file is therefore combersome, this function solves the problem by removing the redundant package name from the path.
#[macro_export]
macro_rules! parent {
    () => {{
        let parent = ::std::path::Path::new(file!())
            .parent()
            .unwrap()
            .components()
            .skip(1);
        ::std::path::PathBuf::from_iter(parent)
    }};
}
