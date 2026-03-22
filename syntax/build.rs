// SPDX-FileCopyrightText: © 2026 Iain Nicol
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{env, fs, path::Path};

use cruet::Inflector;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use ungrammar::{Grammar, NodeData, Rule, TokenData};

fn main() {
    println!("cargo::rerun-if-changed=vb.ungram");

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let ungram: Grammar = include_str!("vb.ungram").parse().unwrap();

    process_nodes(out_dir, ungram.iter().map(|node| &ungram[node]), &ungram);
    process_tokens(out_dir, ungram.tokens().map(|tok| &ungram[tok]));
}

fn process_nodes<'a>(out_dir: &Path, nodes: impl Iterator<Item = &'a NodeData>, ungram: &Grammar) {
    let path = out_dir.join("nodes.rs");

    let head = quote! {
        use std::fmt;

        use crate::{SyntaxKind, ast::{AstNode, SyntaxNode, SyntaxToken, support}};
    };
    let mut stream = head;

    for node in nodes {
        let name = &node.name;
        let ty_name = format_ident!("{}", name);

        let is_enum = if let Rule::Alt(rules) = &node.rule {
            rules.iter().all(|r| matches!(r, Rule::Node(_)))
        } else {
            false
        };
        if is_enum {
            // TODO: refactor to get rid of panic.
            let nodes = match &node.rule {
                Rule::Alt(rules) => rules.iter().map(|r| match r {
                    Rule::Node(node) => &ungram[*node],
                    _ => panic!("impossible"),
                }),
                _ => panic!("not"),
            };
            let node_names = Vec::from_iter(nodes.map(|node| format_ident!("{}", node.name)));
            let generic_enum = quote! {
                #[derive(PartialEq, Eq, Clone, Debug)]
                pub enum #ty_name {
                    #(#node_names(#node_names)),*
                }

                impl AstNode for #ty_name {
                    #[inline]
                    fn can_cast(kind: SyntaxKind) -> bool
                    where
                        Self: Sized,
                    {
                        matches!(kind, #(SyntaxKind::#node_names)|*)
                    }

                    #[inline]
                    fn cast(syntax: SyntaxNode) -> Option<Self>
                    where
                        Self: Sized,
                    {
                        match syntax.kind() {
                            // #(SyntaxKind::#node_names => Some(1)),*
                            #(SyntaxKind::#node_names => Some(#ty_name::#node_names(#node_names { syntax }))),*,
                            // ENUM => Some(Adt::Enum(Enum { syntax })),
                            _ => None,
                        }
                    }

                    #[inline]
                    fn syntax(&self) -> &SyntaxNode {
                        match self {
                            #(#ty_name::#node_names(x) => &x.syntax),*
                        }
                    }
                }

                impl fmt::Display for #ty_name {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        fmt::Display::fmt(self.syntax(), f)
                    }
                }
            };
            stream.extend(generic_enum);
        } else {
            let generic_struct = quote! {
                #[derive(PartialEq, Eq, Clone, Hash)]
                pub struct #ty_name {
                    syntax: SyntaxNode,
                }

                impl AstNode for #ty_name {
                    #[inline]
                    fn kind() -> SyntaxKind {
                        SyntaxKind::#ty_name
                    }

                    #[inline]
                    fn can_cast(kind: SyntaxKind) -> bool
                    where
                        Self: Sized,
                    {
                        kind == Self::kind()
                    }

                    #[inline]
                    fn cast(syntax: SyntaxNode) -> Option<Self>
                    where
                        Self: Sized,
                    {
                        if Self::can_cast(syntax.kind()) {
                            Some(Self { syntax })
                        } else {
                            None
                        }
                    }

                    #[inline]
                    fn syntax(&self) -> &SyntaxNode {
                        &self.syntax
                    }
                }

                impl fmt::Debug for #ty_name {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        fmt::Debug::fmt(&self.syntax, f)
                    }
                }

                impl fmt::Display for #ty_name {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        fmt::Display::fmt(self.syntax(), f)
                    }
                }
            };
            stream.extend(generic_struct);
        };
        // TODO: because of Seq, and Alt and Opt, could more recursive (although we add methods to Node if inside Alt). Rep of Node might also be special (children)?
        let fns: Vec<TokenStream> = match &node.rule {
            ungrammar::Rule::Labeled { label: _, rule: _ } => unimplemented!("labeled"),
            ungrammar::Rule::Node(node) => vec![child_getter(&ungram[*node])],
            ungrammar::Rule::Token(token) => vec![token_getter(&ungram[*token])],
            ungrammar::Rule::Seq(rules) => rules
                .iter()
                .flat_map(|rule| match rule {
                    Rule::Labeled { label: _, rule: _ } => unimplemented!("labeled inside seq"),
                    Rule::Node(node) => vec![child_getter(&ungram[*node])],
                    Rule::Token(token) => vec![token_getter(&ungram[*token])],
                    Rule::Seq(_rules) => unimplemented!("seq inside seq"),
                    Rule::Alt(_rules) => unimplemented!("alt inside seq"),
                    Rule::Opt(rule) => match &**rule {
                        Rule::Labeled { label: _, rule: _ } => {
                            unimplemented!("labeled inside opt inside seq")
                        }
                        Rule::Node(_node) => unimplemented!("node inside opt inside seq"),
                        Rule::Token(token) => vec![token_getter(&ungram[*token])],
                        Rule::Seq(_rules) => unimplemented!("seq inside opt inside seq"),
                        Rule::Alt(_rules) => unimplemented!("alt inside opt inside seq"),
                        Rule::Opt(_rule) => unimplemented!("opt inside opt inside seq"),
                        Rule::Rep(_rule) => unimplemented!("rep inside opt inside seq"),
                    },
                    Rule::Rep(_rule) => unimplemented!("rep inside seq"),
                })
                .collect(),
            ungrammar::Rule::Alt(rules) => rules
                .iter()
                .flat_map(|rule| match rule {
                    Rule::Labeled { label: _, rule: _ } => unimplemented!("labeled inside alt"),
                    // TODO: generate From<Case> for Enum {}.
                    Rule::Node(_node) => [], // Simple enum, can cast and pattern match.
                    Rule::Token(_token) => unimplemented!("token inside alt"),
                    Rule::Seq(_rules) => unimplemented!("seq inside alt"),
                    Rule::Alt(_rules) => unimplemented!("alt inside alt"),
                    Rule::Opt(_rule) => unimplemented!("opt inside alt"),
                    Rule::Rep(_rule) => unimplemented!("rep inside alt"),
                })
                .collect(),
            ungrammar::Rule::Opt(_rule) => unimplemented!("opt"),
            ungrammar::Rule::Rep(rule) => match &**rule {
                Rule::Labeled { label: _, rule: _ } => unimplemented!("labeled inside rep"),
                Rule::Node(node) => vec![children_getter(&ungram[*node])],
                Rule::Token(_token) => unimplemented!("token inside rep"),
                Rule::Seq(_rules) => unimplemented!("seq inside rep"),
                Rule::Alt(_rules) => unimplemented!("alt inside rep"),
                Rule::Opt(_rule) => unimplemented!("opt inside rep"),
                Rule::Rep(_rule) => unimplemented!("rep inside rep"),
            },
        };
        let r#impl = quote! {
            impl #ty_name {
                #(#fns)*
            }
        };
        stream.extend(r#impl);
    }

    write_syn(&path, stream);
}

fn child_getter(node: &NodeData) -> TokenStream {
    let name = &node.name;
    let ty_name = format_ident!("{}", name);
    let fn_name = format_ident!("{}", name.to_snake_case());
    quote! {
        #[inline]
        pub fn #fn_name(&self) -> Option<#ty_name> {
            support::child(&self.syntax)
        }
    }
}

fn token_getter(token: &TokenData) -> TokenStream {
    let name = proper_token_name(&token.name);
    let syntax_kind = format_ident!("{}", name);
    let fn_name = format_ident!("{}_token", name.to_snake_case());
    quote! {
        #[inline]
        pub fn #fn_name(&self) -> Option<SyntaxToken> {
            support::token(&self.syntax, SyntaxKind::#syntax_kind)
        }
    }
}

fn children_getter(node: &NodeData) -> TokenStream {
    let name = &node.name;
    let ty_name = format_ident!("{}", name);
    let fn_name = format_ident!("{}", name.to_snake_case().to_plural());
    quote! {
        #[inline]
        pub fn #fn_name(&self) -> impl Iterator<Item = #ty_name> {
            support::children(&self.syntax)
        }
    }
}

fn process_tokens<'a>(out_dir: &Path, toks: impl Iterator<Item = &'a TokenData>) {
    let path = out_dir.join("tokens.rs");

    let head = quote! {
        use std::fmt;

        use crate::{SyntaxKind, ast::{AstToken, SyntaxToken}};
    };
    let mut stream = head;

    for tok in toks {
        let name = proper_token_name(&tok.name);
        let ty_name = format_ident!("{}", name);
        let generic = quote! {
            #[derive(PartialEq, Eq, Clone, Hash)]
            pub struct #ty_name {
                syntax: SyntaxToken,
            }

            impl AstToken for #ty_name {
                #[inline]
                fn kind() -> SyntaxKind {
                    SyntaxKind::#ty_name
                }

                #[inline]
                fn can_cast(kind: SyntaxKind) -> bool
                where
                    Self: Sized,
                {
                    kind == Self::kind()
                }

                #[inline]
                fn cast(syntax: SyntaxToken) -> Option<Self>
                where
                    Self: Sized,
                {
                    if Self::can_cast(syntax.kind()) {
                        Some(Self { syntax })
                    } else {
                        None
                    }
                }

                #[inline]
                fn syntax(&self) -> &SyntaxToken {
                    &self.syntax
                }
            }

            impl fmt::Debug for #ty_name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    fmt::Debug::fmt(&self.syntax, f)
                }
            }

            impl fmt::Display for #ty_name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    fmt::Display::fmt(self.syntax(), f)
                }
            }
        };
        stream.extend(generic);
    }

    write_syn(&path, stream);
}

fn proper_token_name(tok_name: &str) -> String {
    match tok_name {
        "#ident" => "Ident",
        "," => "Comma",
        "(" => "LParen",
        ")" => "RParen",
        "eol" => "Eol",
        "@string" => "StringLit",
        "Call" => "CallKw",
        "End" => "EndKw",
        "Sub" => "SubKw",
        _ => tok_name,
    }
    .to_string()
}

fn write_syn(path: &Path, stream: proc_macro2::TokenStream) {
    let tree = syn::parse2(stream).unwrap();
    // note this preserves doc comments but not other comments
    let pretty_printed = prettyplease::unparse(&tree);
    fs::write(path, pretty_printed).unwrap();
}
