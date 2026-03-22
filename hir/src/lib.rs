// SPDX-FileCopyrightText: © 2026 Iain Nicol
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use syntax::ast::{self, Expr};

pub struct BasFile {
    // TODO
}

// TODO: lower
pub fn lower(ast: &ast::BasFile) -> BasFile {
    // FIXME: can't go unwrapping things.
    for sub in ast.sub_blocks() {
        let start = sub.sub_stmt().unwrap();
        let name = start.name().unwrap();
        let _name = name.ident_token().unwrap().text();

        let stmts = sub.stmt_list().unwrap();
        for stmt in stmts.stmts() {
            match stmt {
                ast::Stmt::ImplicitCallStmt(implicit_call_stmt) => {
                    let name_ref = implicit_call_stmt.name_ref().unwrap();
                    let _ident = name_ref.ident_token().unwrap().text();
                    // FIXME it's legit for there to be no args
                    let args = implicit_call_stmt.arg_list().unwrap();
                    for arg in args.args() {
                        let expr = arg.expr().unwrap();
                        match expr {
                            Expr::LiteralExpr(literal_expr) => {
                                let str_lit = literal_expr.string_lit_token().unwrap(); // FIXME should assume this could fail, if another type of literal
                                let _str_lit = str_lit.text(); // TODO: need to parse the raw token text. It's quoted.
                            }
                            Expr::BinExpr(_bin_expr) => todo!(), // TODO
                        }
                    }
                }
                ast::Stmt::ExplicitCallStmt(explicit_call_stmt) => {
                    let name_ref = explicit_call_stmt.name_ref().unwrap();
                    let _ident = name_ref.ident_token().unwrap().text();
                    // FIXME it's legit for there to be no parens
                    let _l_paren = explicit_call_stmt.l_paren_token().unwrap().text();
                    let _r_paren = explicit_call_stmt.r_paren_token().unwrap().text();
                    // FIXME it's legit for there to be no args
                    let args = explicit_call_stmt.arg_list().unwrap();
                    for arg in args.args() {
                        let expr = arg.expr().unwrap();
                        match expr {
                            Expr::LiteralExpr(literal_expr) => {
                                let str_lit = literal_expr.string_lit_token().unwrap(); // FIXME should assume this could fail, if another type of literal
                                let _str_lit = str_lit.text(); // TODO: need to parse the raw token text. It's quoted.
                            }
                            Expr::BinExpr(_bin_expr) => todo!(), // TODO
                        } 
                    }
                }
            }
        }

        let end = sub.end_sub_stmt().unwrap();
        _ = end.end_kw_token().unwrap();
        _ = end.sub_kw_token().unwrap();
        _ = end.eol_token().unwrap();
    }

    BasFile {}
}
