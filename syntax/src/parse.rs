// SPDX-FileCopyrightText: © 2026 Iain Nicol
//
// SPDX-License-Identifier: AGPL-3.0-or-later

// TODO: split out generic parser helpers from the vb parser

use cstree::{
    build::GreenNodeBuilder,
    syntax::{ResolvedNode, SyntaxNode},
};
use itertools::{PeekNth, peek_nth};

use crate::{
    SyntaxKind, VisualBasic, ast::{AstNode, BasFile}, lex::{Lexer, Token}
};

// TOOD: define a syntaxkind macro - make this file's code nicer. e.g. s!("(") -> SyntaxKind::LParen

pub fn parse(lex: Lexer) -> BasFile {
    let mut parser = Parser::new(lex);
    parse_bas_file(&mut parser);
    let res = parser.finish();
    BasFile::cast(res).expect("bas file must be bas file")
}

fn parse_bas_file(p: &mut Parser) {
    let mut node = p.start_node(SyntaxKind::BasFile);
    // TODO: it'd be tighter to return tokenkind not syntaxkind
    // TODO: can I automagicaly deal with whitespace? when peeking, when eating etc.
    loop {
        match node.current() {
            SyntaxKind::Eof => return,
            SyntaxKind::Space => node.bump(),
            SyntaxKind::Eol => node.bump(),
            SyntaxKind::SubKw => parse_sub_block(&mut node), // FIXME
            _ => node.err_and_bump(),
        }
    }
}

// TODO: in general: error recovery of parsing
fn parse_sub_block(node: &mut Node<'_, '_>) {
    let mut node = node.start_node(SyntaxKind::SubBlock);
    parse_sub_stmt(&mut node);
    parse_stmt_list(&mut node);
    parse_end_sub_stmt(&mut node);
}

fn parse_sub_stmt(node: &mut Node<'_, '_>) {
    let mut node = node.start_node(SyntaxKind::SubStmt);
    node.eat(SyntaxKind::SubKw);
    node.eat(SyntaxKind::Space);
    parse_name(&mut node);
    parse_param_list(&mut node);
    eol(&mut node);
}

fn parse_stmt_list(node: &mut Node<'_, '_>) {
    let mut node = node.start_node(SyntaxKind::StmtList);
    // FIXME don't loop forever if reach EOF
    while node.current3() != (SyntaxKind::EndKw, SyntaxKind::Space, SyntaxKind::SubKw) {
        parse_stmt(&mut node);
    }
}

fn parse_param_list(node: &mut Node<'_, '_>) {
    let mut node = node.start_node(SyntaxKind::ParamList);
    node.eat(SyntaxKind::LParen);
    node.eat(SyntaxKind::RParen);
}

fn eol(node: &mut Node<'_, '_>) {
    node.eat(SyntaxKind::Eol);
}

// FIXME: should EOF end up in the parse tree? surely not.
fn eol_or_eof(node: &mut Node<'_, '_>) {
    match node.current() {
        SyntaxKind::Eol | SyntaxKind::Eof => node.bump(),
        _ => {}
    }
}

fn parse_name(node: &mut Node<'_, '_>) {
    let mut node = node.start_node(SyntaxKind::Name);
    node.eat(SyntaxKind::Ident);
}

fn parse_name_ref(node: &mut Node<'_, '_>) {
    let mut node = node.start_node(SyntaxKind::NameRef);
    node.eat(SyntaxKind::Ident);
}

fn parse_end_sub_stmt(node: &mut Node<'_, '_>) {
    let mut node = node.start_node(SyntaxKind::EndSubStmt);
    node.eat(SyntaxKind::EndKw);
    node.eat(SyntaxKind::Space);
    node.eat(SyntaxKind::SubKw);
    eol_or_eof(&mut node);
}

fn parse_stmt(node: &mut Node<'_, '_>) {
    match node.current() {
        SyntaxKind::Eof => (),
        SyntaxKind::Space => node.bump(),
        SyntaxKind::Eol => node.bump(),
        SyntaxKind::Ident => parse_implicit_call_stmt(node),
        SyntaxKind::CallKw => parse_explicit_call_stmt(node),
        _ => node.err_and_bump(),
    }
}

fn parse_implicit_call_stmt(node: &mut Node<'_, '_>) {
    let mut node = node.start_node(SyntaxKind::ImplicitCallStmt);
    parse_name_ref(&mut node);
    // TODO: support X.Y
    // TODO: exit if eol or eof (no arguments)
    node.eat(SyntaxKind::Space);
    parse_arg_list(&mut node);
    eol(&mut node);
}

fn parse_arg_list(node: &mut Node<'_, '_>) {
    let mut node = node.start_node(SyntaxKind::ArgList);
    // TODO: parse 0?
    // TODO: parse n
    parse_arg(&mut node);
}

fn parse_arg(node: &mut Node<'_, '_>) {
    let mut node = node.start_node(SyntaxKind::Arg);
    parse_expr(&mut node);
}

// TODO: other exprs
fn parse_expr(node: &mut Node<'_, '_>) {
    assert!(try_parse_literal(node))
}

fn try_parse_literal(node: &mut Node<'_, '_>) -> bool {
    if node.current() != SyntaxKind::StringLit {
        return false;
    }
    let mut node = node.start_node(SyntaxKind::LiteralExpr);
    node.eat(SyntaxKind::StringLit);
    true
}

fn parse_explicit_call_stmt(node: &mut Node<'_, '_>) {
    let mut node = node.start_node(SyntaxKind::ExplicitCallStmt);
    node.eat(SyntaxKind::CallKw);
    node.eat(SyntaxKind::Space);
    parse_name_ref(&mut node);
    node.eat(SyntaxKind::LParen);
    parse_arg_list(&mut node);
    node.eat(SyntaxKind::RParen);
    eol(&mut node);
}

pub struct Parser<'a> {
    lexer: PeekNth<Lexer<'a>>,
    builder: GreenNodeBuilder<'static, 'static, VisualBasic>,
}

impl<'a> Parser<'a> {
    pub fn new(lex: Lexer<'a>) -> Parser<'a> {
        Self {
            lexer: peek_nth(lex),
            builder: GreenNodeBuilder::new(),
        }
    }

    pub fn finish(mut self) -> ResolvedNode<VisualBasic> {
        let _next_tok = self.lexer.next().map(|(t, _)| t);
        // TODO: enable once properly parsing
        // assert!(next_tok.is_none() || next_tok == Some(Token::Eof));

        let (tree, cache) = self.builder.finish();
        let interner = cache.unwrap().into_interner().unwrap();

        SyntaxNode::new_root_with_resolver(tree, interner)
    }

    pub fn current(&mut self) -> SyntaxKind {
        self.lexer
            .peek()
            .map(|(t, _)| t)
            .unwrap_or(&Token::Eof)
            .to_syntax_kind()
    }

    pub fn current2(&mut self) -> (SyntaxKind, SyntaxKind) {
        let tok = self
            .lexer
            .peek()
            .map(|(t, _)| t)
            .unwrap_or(&Token::Eof)
            .to_syntax_kind();
        let tok1 = self
            .lexer
            .peek_nth(1)
            .map(|(t, _)| t)
            .unwrap_or(&Token::Eof)
            .to_syntax_kind();
        (tok, tok1)
    }

    // FIXME it's like current3 isn't working at the end of the file.
    // oddly didn't work with itertools either.
    pub fn current3(&mut self) -> (SyntaxKind, SyntaxKind, SyntaxKind) {
        let tok = self
            .lexer
            .peek()
            .map(|(t, _)| t)
            .unwrap_or(&Token::Eof)
            .to_syntax_kind();
        let tok1 = self
            .lexer
            .peek_nth(1)
            .map(|(t, _)| t)
            .unwrap_or(&Token::Eof)
            .to_syntax_kind();
        let tok2 = self
            .lexer
            .peek_nth(2)
            .map(|(t, _)| t)
            .unwrap_or(&Token::Eof)
            .to_syntax_kind();
        (tok, tok1, tok2)
    }

    pub fn is_at(&mut self, kind: SyntaxKind) -> bool {
        self.current() == kind
    }

    pub fn try_eat(&mut self, kind: SyntaxKind) -> bool {
        if self.is_at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub fn eat(&mut self, kind: SyntaxKind) {
        if !self.try_eat(kind) {
            self.err_and_bump();
        }
    }

    pub fn bump(&mut self) {
        let tok = self.lexer.next().unwrap_or((Token::Eof, ""));
        self.builder.token(tok.0.to_syntax_kind(), tok.1);
    }

    // FIXME: due to ownership I'm having to duplicate everything between parser and node
    // Can I just return another parser but with a shorter lifetime?
    // Alternatively I want to distinguish between Parser and TreeBuilder.
    #[must_use]
    pub fn start_node<'b>(&'b mut self, kind: SyntaxKind) -> Node<'a, 'b> {
        self.builder.start_node(kind);
        Node { parser: self }
    }

    pub fn err_and_bump(&mut self) {
        let mut node = self.start_node(SyntaxKind::Error);
        node.bump()
    }
}

pub struct Node<'a, 'b> {
    parser: &'b mut Parser<'a>,
}

impl<'a, 'b> Node<'a, 'b> {
    pub fn current(&mut self) -> SyntaxKind {
        self.parser.current()
    }

    pub fn current2(&mut self) -> (SyntaxKind, SyntaxKind) {
        self.parser.current2()
    }

    pub fn current3(&mut self) -> (SyntaxKind, SyntaxKind, SyntaxKind) {
        self.parser.current3()
    }

    // FIXME: should this eat tokens instead? that's stricter.
    pub fn eat(&mut self, kind: SyntaxKind) {
        self.parser.eat(kind);
    }

    pub fn bump(&mut self) {
        self.parser.bump();
    }

    #[must_use]
    pub fn start_node<'c>(&'c mut self, kind: SyntaxKind) -> Node<'a, 'c> {
        self.parser.start_node(kind)
    }

    pub fn err_and_bump(&mut self) {
        self.parser.err_and_bump();
    }
}

impl<'a, 'b> Drop for Node<'a, 'b> {
    fn drop(&mut self) {
        self.parser.builder.finish_node();
    }
}
