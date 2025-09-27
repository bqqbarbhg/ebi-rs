use crate::{ast::*, *};

struct Parser<'a> {
    tokens: &'a mut dyn Iterator<Item = Token>,
    token: Token,
    errors: &'a dyn Errors,
}

impl<'a> Parser<'a> {
    pub fn new(errors: &'a dyn Errors, tokens: &'a mut dyn Iterator<Item = Token>) -> Parser<'a> {
        Parser {
            tokens,
            token: Token::error(),
            errors,
        }
    }

    fn advance(&mut self) -> Token {
        let token = match self.tokens.next() {
            Some(token) => token,
            None => Token::end(),
        };
        std::mem::replace(&mut self.token, token)
    }

    fn accept(&mut self, tk: TokenKind) -> Option<Token> {
        if self.token.kind == tk {
            Some(self.advance())
        } else {
            None
        }
    }

    fn skip_newlines(&mut self) {
        while self.accept(TokenKind::Newline).is_some() {}
    }

    fn parse_atom(&mut self) -> Option<Ast> {
        match self.token.kind {
            TokenKind::Ident => Some(Ast::Name(self.advance())),
            _ => None,
        }
    }

    fn parse_term(&mut self) -> Option<Ast> {
        let mut lhs = self.parse_atom()?;
        loop {
            match self.token.kind {
                TokenKind::Add => {
                    let op = self.advance();
                    let rhs = self.parse_atom()?;
                    lhs = Ast::Binop(op, Box::new(lhs), Box::new(rhs));
                },
                _ => { break },
            }
        }
        Some(lhs)
    }

    fn parse_expr(&mut self) -> Option<Ast> {
        self.parse_term()
    }

    fn finish_class(&mut self, tok: Token) -> Option<Ast> {
        let Some(name) = self.accept(TokenKind::Ident) else {
            error!(self, &self.token, "expected name for class");
            return None
        };

        self.skip_newlines();
        if self.accept(TokenKind::BraceOpen).is_none() {
            error!(self, &self.token, "expected '{{' following a class declaration");
            return None
        };

        let mut decls = Vec::new();

        self.skip_newlines();
        while self.accept(TokenKind::BraceClose).is_none() {
            self.skip_newlines();
            if self.accept(TokenKind::End).is_some() {
                error!(self, &name, "unclosed class");
                break;
            }

            if let Some(decl) = self.parse_decl() {
                decls.push(decl);
            } else {
                error!(self, &self.token, "expected a declaration");
                self.recover_decl();
            }
            self.skip_newlines();
        };

        Some(Ast::ClassDecl(name, decls))
    }

    fn parse_decl(&mut self) -> Option<Ast> {
        match self.token.kind {
            TokenKind::KeywordClass | TokenKind::KeywordStruct => {
                let token = self.advance();
                self.finish_class(token)
            }
            _ => self.parse_expr(),
        }
    }

    fn recover_decl(&mut self) {
        loop {
            match self.token.kind {
                TokenKind::Newline | TokenKind::End => {
                    self.advance();
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn parse(&mut self) -> Ast {
        self.advance();

        let mut decls = Vec::new();

        self.skip_newlines();
        while self.accept(TokenKind::End).is_none() {
            self.skip_newlines();
            if let Some(decl) = self.parse_decl() {
                decls.push(decl);
            } else {
                self.recover_decl()
            }
            self.skip_newlines();
        }

        Ast::Root(decls)
    }
}

impl<'a> Errors for Parser<'a> {
    fn push(&self, int_loc: &InternalLocation, loc: &dyn Locatable, message: String, context: Vec<String>) {
        self.errors.push(int_loc, loc, message, context);
    }
}

pub fn parse(errors: &dyn Errors, tokens: impl Iterator<Item = Token>) -> Ast {
    let mut tokens = tokens;
    let mut parser = Parser::new(errors, &mut tokens);
    parser.parse()
}
