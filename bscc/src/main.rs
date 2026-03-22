// SPDX-FileCopyrightText: © 2026 Iain Nicol
//
// SPDX-License-Identifier: AGPL-3.0-or-later

// TODO: man page for bscc

// TODO: move crates into a crates subdir? or compiler versus library? or src versus app versus stdlib? update workspace.

// TODO: cargo-about for license file for binaries (+ manually append llvm...)

// TODO: stop expect()ing and panic!()ing and unwrap()ing.
// TODO: support comments in lex and parse
// TODO: fix rust warnings
// TODO: use salsa for semantic analysis
// TODO: language server (as a different app)
// TODO: unit tests
// TODO: integration tests, using vb6.exe, on macos (wine), linux (wine), and windows

#![allow(dead_code)]

use std::sync::LazyLock;
use std::{env, ffi::OsString, path::Path};

use syntax::{lex, parse};
use target_lexicon::{Triple, triple};

use crate::argparse::{Options, ParseError};

mod argparse;

fn main() {
    let mut args = env::args_os();
    let process_name = args.next().expect("unknown process name");
    match argparse::parse(args) {
        Ok(r) if r.options.help => println!(
            "Brainsick compiler 
            
Usage: {} [options] file...

Options:
  --help           Show usage
  --version        Show version and copyright
  -O0, -O1, -O2    Set optimization level
  -S -emit-llvm
  -o file          Set output file",
            process_name.display()
        ),
        Ok(r) if r.options.version => {
            println!("bscc version {}", env!("CARGO_PKG_VERSION"));
            println!("Copyright © 2026 Iain Nicol");
            println!("This is free software, licensed under AGPL-3.0-or-later.");
            println!("There is NO warranty, not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.");
        },
        Ok(r) => {
            if r.positional_arguments.is_empty() {
                eprintln!("Input file required")
            } else {
                main_inner(r.options, &r.positional_arguments)
            }
        }
        Err(e) => match e {
            ParseError::MissingArgumentToOption(o) => {
                eprintln!("Missing argument to option {}", o.display())
            }
            ParseError::InvalidOption(o) => eprintln!("Invalid option {}", o.display()),
        },
    }
}

#[cfg(windows)]
const DEFAULT_OUTPUT_FILE: &str = "a.exe";

#[cfg(not(windows))]
const DEFAULT_OUTPUT_FILE: &str = "a.out";

// FIXME don't hard code architecture! especially not for linux.
// TODO: get default triple from llvm (but downgrade macos target version if can).
#[cfg(target_os = "macos")]
static DEFAULT_TARGET_TRIPLE: LazyLock<Triple> =
    LazyLock::new(|| triple!("arm64-apple-macosx14.0.0"));

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
static DEFAULT_TARGET_TRIPLE: LazyLock<Triple> =
    LazyLock::new(|| triple!("aarch64-linux-gnu"));

#[cfg(all(target_os = "linux", not(target_arch = "aarch64")))]
static DEFAULT_TARGET_TRIPLE: LazyLock<Triple> =
    LazyLock::new(|| triple!("x86_64-linux-gnu"));

#[cfg(target_os = "windows")]
static DEFAULT_TARGET_TRIPLE: LazyLock<Triple> =
    LazyLock::new(|| triple!("x86_64-pc-windows-msvc"));


fn main_inner(options: Options, input_files: &[impl AsRef<Path>]) {
    let srcs: Vec<(&Path, String)> = input_files
        .iter()
        .map(|f| (f.as_ref(), lex::decode_file(f.as_ref())))
        .collect();
    let asts: Vec<_> = srcs
        .iter()
        .map(|(path, src)| (path, parse::parse(lex::lex_str(src))))
        .collect();
    let hirs: Vec<_> = asts
        .iter()
        .map(|(path, ast)| (path, hir::lower(ast)))
        .collect();
    let triple = &*DEFAULT_TARGET_TRIPLE;
    let obj_files = codegen::codegen(triple, &hirs);
    let exe_file = &options
        .output_file
        .unwrap_or(OsString::from(DEFAULT_OUTPUT_FILE));
    let exe_file = Path::new(exe_file);
    link::link(triple, &obj_files, exe_file);
}
