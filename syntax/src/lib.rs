// SPDX-FileCopyrightText: © 2026 Iain Nicol
//
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod ast;
pub mod lex;
pub mod parse;

#[derive(Debug, Clone, Copy, PartialEq, Eq, cstree::Syntax)]
#[repr(u32)]
pub enum SyntaxKind {
    Eof,
    // Tokens
    Space,
    Eol,
    CallKw,
    EndKw,
    SubKw,
    #[static_text("(")]
    LParen,
    #[static_text(")")]
    RParen,
    #[static_text(",")]
    Comma,
    Ident,
    StringLit,
    // Nodes
    Error,
    BasFile,
    SubBlock,
    SubStmt,
    EndSubStmt,
    Name,
    NameRef,
    ParamList,
    Param,
    ImplicitCallStmt,
    ExplicitCallStmt,
    ArgList,
    Arg,
    LiteralExpr,
    BinExpr,
    ModuleItem,
    StmtList,
    Stmt,
}

pub type VisualBasic = SyntaxKind;
