# README
This repository contains a single package, `rust-utils`, which contains a few random utilities that I've used in various Rust projects.

## Continuous Integration
This repository includes a script that runs various checks on the code, i.e. it:
1. performs static analysis of the source code.
1. Builds the documentation.
1. And checks the formatting of the code.

It can launched with the `cargo ci` command.

It's implemented in Rust following the `xtasks` pattern, as explained by [these](https://matklad.github.io/2018/01/03/make-your-own-make.html) [two](https://blog.rng0.io/running-rust-tasks-with-xtask-and-xtaskops/) articles. However, contrary to the standard, this script is part of the main package instead of being implemented as a separate one. This is because the main package includes things that are used by the CI script, so separating them makes less sense.
