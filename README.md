# silver
A simple, efficient, cross-platform text editor for the terminal.

## Building
`cargo build` will compile and run the executable in development mode.

`./install.py` (currently functions only on \*nix) will compile in release mode, configure syntax highlighting, and add `silver` to $PATH.

This requires the [nightly](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html) toolchain.

## Running
`cargo run <args?>` will compile and run the executable in development mode.

If previously compiled in release mode and added to $PATH, simply invoke `silver` on a file of your choice!

## Cleaning Up
`cargo clean [OPTIONS]` will clean the target directory and all binary files.
