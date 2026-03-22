// SPDX-FileCopyrightText: © 2026 Iain Nicol
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fs, path::Path};

use encoding_rs::WINDOWS_1252;
use logos::Logos;

use crate::SyntaxKind;

pub struct Lexer<'a> {
    logos: logos::Lexer<'a, Token>,
    finished: bool,
}

impl<'a> Iterator for Lexer<'a> {
    // TODO: named struct
    type Item = (Token, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        Some(match self.logos.next() {
            Some(r) => match r {
                Ok(tok) => (tok, self.logos.slice()),
                Err(_) => (Token::Unknown, self.logos.slice()),
            },
            None => {
                self.finished = true;
                (Token::Eof, "")
            }
        })
    }
}

pub fn decode_file(file: &Path) -> String {
    // TODO: -finput-charset.
    let bytes = fs::read(file).expect("failed to read file");
    match String::from_utf8(bytes) {
        Ok(str) => str,
        Err(non_utf8_bytes) => {
            let encoding: &encoding_rs::Encoding = WINDOWS_1252;
            // TODO: propogate err
            let (str, _, _err) = encoding.decode(non_utf8_bytes.as_bytes());
            str.into_owned()
        }
    }
}

pub fn lex_str<'a>(source: &'a str) -> Lexer<'a> {
    Lexer {
        logos: Token::lexer(source),
        finished: false,
    }
}

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[regex(" +")]
    Space,
    #[regex("(\r\n|\n)+")]
    Eol,
    #[regex("Call", ignore(case))]
    CallKw,
    #[regex("End", ignore(case))]
    EndKw,
    #[regex("Sub", ignore(case))]
    SubKw,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(",")]
    Comma,
    #[regex("[A-Za-z]+")]
    Ident,
    #[regex(r#""[^"\r\n]*""#)]
    StringLit,
    Unknown,
    Eof,
}

impl Token {
    pub fn to_syntax_kind(&self) -> SyntaxKind {
        match self {
            Token::Space => SyntaxKind::Space,
            Token::Eol => SyntaxKind::Eol,
            Token::CallKw => SyntaxKind::CallKw,
            Token::EndKw => SyntaxKind::EndKw,
            Token::SubKw => SyntaxKind::SubKw,
            Token::LParen => SyntaxKind::LParen,
            Token::RParen => SyntaxKind::RParen,
            Token::Comma => SyntaxKind::Comma,
            Token::Ident => SyntaxKind::Ident,
            Token::StringLit => SyntaxKind::StringLit,
            Token::Unknown => SyntaxKind::Error,
            Token::Eof => SyntaxKind::Eof,
        }
    }
}