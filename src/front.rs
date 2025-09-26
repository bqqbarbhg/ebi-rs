mod tokenizer;
mod parser;

use crate::errors::*;
pub use tokenizer::tokenize;

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
    Newline,
    End,
}

#[derive(Debug, Clone)]
pub struct Token {
    kind: TokenKind,
    span: SourceSpan,
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        self.kind
    }
    pub fn span(&self) -> SourceSpan {
        self.span
    }
}

pub enum BinaryOp {
    Add,
}

pub enum ExprKind {
    Error,
    Identifier,
    BinaryOp(BinaryOp, Expr, Expr),
}

pub struct Expr {
    kind: Box<ExprKind>,
    span: SourceSpan,
}

impl Expr {
    pub fn kind(&self) -> &ExprKind {
        &self.kind
    }
}

pub enum StmtKind {
    Error,
    If(Expr, Stmt),
}

pub struct Stmt {
    kind: Box<StmtKind>,
    span: SourceSpan,
}

impl Stmt {
    pub fn kind(&self) -> &StmtKind {
        &self.kind
    }
}


impl Locatable for SourceSpan {
    fn source_span(&self, _: &dyn Locator) -> SourceSpan {
        *self
    }
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

