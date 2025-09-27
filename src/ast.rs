use bumpalo::Bump;
use self_cell::self_cell;

use crate::*;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum TokenKind {
    Error,
    Ident,
    Integer,
    Float,
    Assign,
    Equals,
    Greater,
    Less,
    GreaterEquals,
    LessEquals,
    BraceOpen,
    BraceClose,
    Add,
    Newline,
    End,
    KeywordClass,
    KeywordStruct,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

impl Token {
    pub fn error() -> Token {
        Token {
            kind: TokenKind::Error,
            span: SourceSpan::unknown(),
        }
    }
    pub fn end() -> Token {
        Token {
            kind: TokenKind::End,
            span: SourceSpan::unknown(),
        }
    }
}

pub enum BinaryOp {
    Add,
}

#[derive(Clone, Debug)]
pub enum Ast<'a> {
    Error(Token),
    Root(&'a [Ast<'a>]),
    ClassDecl(Token, Token, &'a [Ast<'a>]),
    Name(Token),
    Binop(Token, &'a Ast<'a>, &'a Ast<'a>),
}

impl Ast<'_> {
    pub fn error(token: Token) -> Ast<'static> {
        Ast::Error(token)
    }
}

self_cell!(
    struct AstCell {
        owner: Bump,
        #[covariant]
        dependent: Ast,
    }

    impl { Debug }
);

pub struct AstRoot {
    cell: AstCell,
}

impl AstRoot {
    pub fn new<F: FnOnce(&Bump) -> Ast>(f: F) -> AstRoot {
        let bump = Bump::new();
        AstRoot { cell: AstCell::new(bump, f) }
    }

    pub fn root<'a>(&'a self) -> Ast<'a> {
        self.cell.borrow_dependent().clone()
    }
}

pub enum ExprKind {
    Error,
    Identifier,
    BinaryOp(BinaryOp, Expr, Expr),
}

pub struct Expr {
    pub kind: Box<ExprKind>,
    pub span: SourceSpan,
}

pub enum StmtKind {
    Error,
    If(Expr, Stmt),
}

pub struct Stmt {
    pub kind: Box<StmtKind>,
    pub span: SourceSpan,
}

pub enum DeclKind {
    Error,
}

pub struct Decl {
    pub kind: Box<DeclKind>,
    pub span: SourceSpan,
}

impl Decl {
    pub fn new(kind: DeclKind, span: SourceSpan) -> Decl {
        Decl {
            kind: Box::new(kind),
            span,
        }
    }
    pub fn error(span: SourceSpan) -> Decl {
        Decl::new(DeclKind::Error, span)
    }
}

pub struct Root {
    pub decls: Vec<Decl>,
}

impl Locatable for Token {
    fn source_span(&self, _: &dyn Locator) -> SourceSpan {
        self.span
    }
}

impl Locatable for Expr {
    fn source_span(&self, _: &dyn Locator) -> SourceSpan {
        self.span
    }
}

impl Locatable for Stmt {
    fn source_span(&self, _: &dyn Locator) -> SourceSpan {
        self.span
    }
}
