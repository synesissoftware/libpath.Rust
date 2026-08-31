# libpath.Rust <!-- omit in toc -->

Path parsing library, for Rust

![Language](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
[![License](https://img.shields.io/badge/License-BSD_3--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)
[![Crates.io](https://img.shields.io/crates/v/libpath.svg)](https://crates.io/crates/libpath)
[![GitHub release](https://img.shields.io/github/v/release/synesissoftware/libpath.Rust.svg)](https://github.com/synesissoftware/libpath.Rust/releases/latest)
![MSRV](https://img.shields.io/badge/MSRV-1.74-lightgrey)
[![CI](https://github.com/synesissoftware/libpath.Rust/actions/workflows/ci.yml/badge.svg)](https://github.com/synesissoftware/libpath.Rust/actions/workflows/ci.yml)
[![docs.rs](https://docs.rs/libpath/badge.svg)](https://docs.rs/libpath)


## Table of Contents <!-- omit in toc -->

- [Introduction](#introduction)
- [Installation](#installation)
- [Components](#components)
  - [Common result type](#common-result-type)
  - [Unix path classification](#unix-path-classification)
  - [Windows path classification](#windows-path-classification)
- [Examples](#examples)
- [Project Information](#project-information)
  - [Where to get help](#where-to-get-help)
  - [Contribution guidelines](#contribution-guidelines)
  - [Dependencies](#dependencies)
    - [Efferent (fan-out)](#efferent-fan-out)
      - [Runtime Dependencies](#runtime-dependencies)
      - [Build Dependencies](#build-dependencies)
      - [Development Dependencies](#development-dependencies)
    - [Afferent (fan-in)](#afferent-fan-in)
  - [Related projects](#related-projects)
  - [License](#license)


## Introduction

**libpath.Rust** is a small Rust library for classifying Unix-like and
Windows path syntax. It reports path components as positions into the
original input string, without accessing the file system.


## Installation

Reference **libpath** from **Cargo.toml**:

```toml
libpath = { version = "0.0.3" }
```

The repository retains **Cargo.lock** to make local and CI verification
reproducible. Release and validation commands use `--locked`.


## Components

### Common result type

`libpath::util::common::ClassificationResult` describes the positions of
the input, root, directory, entry name, stem, extension, and related path
parts. Its fields use `fastparse::fastparse::types::PositionalSlice`.


### Unix path classification

`libpath::util::unix::path_classify()` classifies Unix-like paths and
returns `libpath::util::unix::Classification` together with a
`ClassificationResult`. The module also defines the parsing flag constants
in `classification_flags`.


### Windows path classification

`libpath::util::windows::path_classify()` classifies paths using forward
or backward separators, drive-letter roots and relative drive paths, and
the currently implemented tilde-rooted form. It returns the Windows
`Classification` and a `ClassificationResult`.


## Examples

There are no maintained example programs yet. The `libver` binary is a
deliberately retained scratch utility that prints the package name and
version:

```text
cargo run --bin libver
```

It is a repository utility rather than an example of the public path API.


## Project Information

### Where to get help

[GitHub Page](https://github.com/synesissoftware/libpath.Rust "GitHub Page")


### Contribution guidelines

Defect reports, feature requests, and pull requests are welcome on
https://github.com/synesissoftware/libpath.Rust.


### Dependencies

#### Efferent (fan-out)

Libraries upon which **libpath.Rust** depends:

##### Runtime Dependencies

* [**FastParse.Rust**](https://github.com/synesissoftware/FastParse.Rust) —
  provides the public `PositionalSlice` result type;


##### Build Dependencies

None.


##### Development Dependencies

* [**test_help-rs**](https://github.com/synesissoftware/test_help-rs) —
  declared for repository test support;


#### Afferent (fan-in)

No downstream consumers are currently recorded.


### Related projects

* [**libpath**](https://github.com/synesissoftware/libpath);
* [**libpath.Go**](https://github.com/synesissoftware/libpath.Go);
* [**libpath.Ruby**](https://github.com/synesissoftware/libpath.Ruby);
* [**recls.Rust**](https://github.com/synesissoftware/recls.Rust);


### License

**libpath.Rust** is released under the 3-clause BSD license. See
[LICENSE](./LICENSE) for details.


<!-- ########################### end of file ########################### -->

