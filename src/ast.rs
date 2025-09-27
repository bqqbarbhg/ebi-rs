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
pub enum Ast {
    Error(Token),
    Root(Vec<Ast>),
    ClassDecl(Token, Vec<Ast>),
    Name(Token),
    Binop(Token, Box<Ast>, Box<Ast>),
}

impl Ast {
    pub fn error(token: Token) -> Ast {
        Ast::Error(token)
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
