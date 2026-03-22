// SPDX-FileCopyrightText: © 2026 Iain Nicol
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{SyntaxKind, VisualBasic};

mod generated {
    pub mod nodes {
        include!(concat!(env!("OUT_DIR"), "/nodes.rs"));
    }

    pub mod tokens {
        include!(concat!(env!("OUT_DIR"), "/tokens.rs"));
    }
}

pub use generated::nodes::*;
pub use generated::tokens::*;

pub type SyntaxNode = cstree::syntax::ResolvedNode<VisualBasic>;
pub type SyntaxToken = cstree::syntax::ResolvedToken<VisualBasic>;

pub trait AstNode {
    fn kind() -> SyntaxKind {
        // FIXME: why don't we just return syntax.kind
        // (a) in general -- assert it's the same
        //
        // (b) for enums (dynamic kinds). like it is not fine
        // to define it as such?
        panic!("Node has a dynamic kind")
    }

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;
}

pub trait AstToken {
    fn kind() -> SyntaxKind;

    fn can_cast(token: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxToken;

    fn text(&self) -> &str {
        self.syntax().text()
    }
}

mod support {
    use super::{AstNode, SyntaxKind, SyntaxNode, SyntaxToken};

    pub(super) fn child<N: AstNode>(parent: &SyntaxNode) -> Option<N> {
        parent.children().cloned().find_map(N::cast)
    }

    // TODO work out why in cstree sexpr example the ret type has: + '_
    pub(super) fn children<N: AstNode>(parent: &SyntaxNode) -> impl Iterator<Item = N> {
        parent.children().cloned().filter_map(N::cast)
    }

    #[inline]
    pub(super) fn token(parent: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxToken> {
        parent
            .children_with_tokens()
            .filter_map(|x| x.into_token())
            .find(|&it| it.kind() == kind)
            .cloned()
    }
}
