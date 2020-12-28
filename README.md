# cargo-pyinit
Cargo extension to make a Rust crate into a hybrid Python/Rust library.

This crate is based on PyO3 https://github.com/PyO3/pyo3

## Prerequisites

You will need:

* Python 3 (https://www.python.org/)
* setuptools_rust (https://github.com/PyO3/setuptools-rust#pyprojecttoml)
* The Python toml library.

setuptools_rust can be installed with pip:

```bash
pip install setuptools_rust
```

And toml can be installed like this:
```bash
pip install toml
```

## Installation

From github:
```bash
git clone https://github.com/rexlunae/cargo-pyinit.git
cd cargo-pyinit
cargo install --path .
```

From cargo:
```bash
cargo install pyinit
```

## Usage

pyinit can only be used to create or modify a library crate.

From inside a source directory:
```bash
cd my-create-directory
cargo pyinit
```

If the directory is already a cargo crate, it will convert it into a hybrid crate suitable for use with python.  If the directory is empty, it will create a new library crate with the same name as the directory it's in.

From outside a directory:
```bash
cargo pyinit crate-name
```

This will create the directory with the given name, if needed, and either create a new crate or modify an existing crate.

Unlike the example code for PyO3, pyinit creates a new file (pylib.rs) that becomes the base of the library, which imports everything from the old lib.rs (or whatever it was previously called).  This separates the Python interface definition somewhat, but it also means that some items that are only allowed at the top level of the crate such as feature flags be moved manually.  When creating a new crate, the result will work out of the box with either `cargo build` or the Python setup tools, however, if you are modifying an existing crate, you will probably have to spend some time manually moving things into the pylib.rs from your lib.rs.  You will need to build the Python interface manually as well.  This does not happen automatically.

The resulting hybrid crate has a setup.py, which can be installed in Python with either `python setup.py develop` for a development version, or `python setup.py install` for release.  You can also use `pip install .`

