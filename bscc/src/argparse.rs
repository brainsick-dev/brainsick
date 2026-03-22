// SPDX-FileCopyrightText: © 2026 Iain Nicol
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ffi::OsString;

#[derive(Debug)]
pub struct ParseResult {
    pub options: Options,
    pub positional_arguments: Vec<OsString>,
}

#[derive(Debug, Default)]
pub struct Options {
    pub help: bool,
    pub version: bool,
    pub optimisation_level: u32,
    pub output_file: Option<OsString>,
}

#[derive(Debug)]
pub enum ParseError {
    MissingArgumentToOption(OsString),
    InvalidOption(OsString),
}

pub fn parse(
    mut args: impl std::iter::Iterator<Item = OsString>,
) -> Result<ParseResult, ParseError> {
    let mut ret = ParseResult {
        options: Options::default(),
        positional_arguments: vec![],
    };
    while let Some(arg) = args.next() {
        if arg.as_encoded_bytes().starts_with(b"-") {
            let option = arg;
            if option == "--" {
                ret.positional_arguments.extend(args);
                break;
            } else if option == "--help" {
                ret.options.help = true;
                return Ok(ret);
            } else if option == "--version" {
                // FIXME: --help should take precedence even if later.
                ret.options.version = true;
                return Ok(ret);
            } else if option == "-O0" {
                ret.options.optimisation_level = 0
            } else if option == "-O1" {
                ret.options.optimisation_level = 1
            } else if option == "-O2" {
                ret.options.optimisation_level = 2
            } else if option == "-o" {
                if let Some(arg) = args.next() {
                    ret.options.output_file = Some(arg)
                } else {
                    return Err(ParseError::MissingArgumentToOption(option));
                }
            } else if option.as_encoded_bytes().starts_with(b"-o") {
                // SAFETY: we start with valid UTF-8. We then remove two whole (ASCII) characters, leaving us with valid UTF-8.
                let arg = unsafe {
                    OsString::from_encoded_bytes_unchecked(option.as_encoded_bytes()[2..].to_vec())
                };
                ret.options.output_file = Some(arg)
            // TODO: --output file, --output=file
            } else {
                return Err(ParseError::InvalidOption(option));
            }
        } else {
            ret.positional_arguments.push(arg);
        }
    }
    Ok(ret)
}
