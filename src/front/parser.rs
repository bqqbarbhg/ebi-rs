use bumpalo::Bump;

use crate::{ast::*, *};

struct Parser<'a, 'b: 'a> {
    tokens: &'a mut dyn Iterator<Item = Token>,
    token: Token,
    errors: &'a dyn Errors,
    bump: &'b Bump,
    temp_lists: Vec<Vec<Ast<'b>>>,
}

impl<'a, 'b> Parser<'a, 'b> {
    pub fn new(errors: &'a dyn Errors, tokens: &'a mut dyn Iterator<Item = Token>, bump: &'b Bump) -> Parser<'a, 'b> {
        Parser {
            tokens,
            token: Token::error(),
            errors,
            bump,
            temp_lists: Vec::new(),
        }
    }

    fn push(&self, ast: Ast<'b>) -> &'b Ast<'b> {
        self.bump.alloc(ast)
    }

    fn push_n(&self, ast: &[Ast<'b>]) -> &'b [Ast<'b>] {
        self.bump.alloc_slice_clone(ast)
    }

    fn begin_list(&mut self) -> Vec<Ast<'b>> {
        match self.temp_lists.pop() {
            Some(list) => list,
            None => Vec::new(),
        }
    }

    fn push_list(&mut self, list: Vec<Ast<'b>>) -> &'b [Ast<'b>] {
        let result = self.push_n(&list);
        let mut list = list;
        list.clear();
        self.temp_lists.push(list);
        result
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

    fn parse_atom(&mut self) -> Option<Ast<'b>> {
        match self.token.kind {
            TokenKind::Ident => Some(Ast::Name(self.advance())),
            _ => None,
        }
    }

    fn parse_term(&mut self) -> Option<Ast<'b>> {
        let mut lhs = self.parse_atom()?;
        loop {
            match self.token.kind {
                TokenKind::Add => {
                    let op = self.advance();
                    let rhs = self.parse_atom()?;
                    let l = self.push(lhs);
                    let r = self.push(rhs);
                    lhs = Ast::Binop(op, l, r);
                },
                _ => { break },
            }
        }
        Some(lhs)
    }

    fn parse_expr(&mut self) -> Option<Ast<'b>> {
        self.parse_term()
    }

    fn finish_class(&mut self, kw: Token) -> Option<Ast<'b>> {
        let Some(name) = self.accept(TokenKind::Ident) else {
            error!(self, &self.token, "expected name for class");
            return None
        };

        self.skip_newlines();
        if self.accept(TokenKind::BraceOpen).is_none() {
            error!(self, &self.token, "expected '{{' following a class declaration");
            return None
        };

        let mut decls = self.begin_list();

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

        Some(Ast::ClassDecl(kw, name, self.push_list(decls)))
    }

    fn parse_decl(&mut self) -> Option<Ast<'b>> {
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

    fn parse(&mut self) -> Ast<'b> {
        self.advance();

        let mut decls = self.begin_list();

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

        Ast::Root(self.push_list(decls))
    }
}

impl<'a, 'b> Errors for Parser<'a, 'b> {
    fn push(&self, int_loc: &InternalLocation, loc: &dyn Locatable, message: String, context: Vec<String>) {
        self.errors.push(int_loc, loc, message, context);
    }
}

pub fn parse(errors: &dyn Errors, tokens: impl Iterator<Item = Token>) -> AstRoot {
    AstRoot::new(|bump| {
        let mut tokens = tokens;
        let mut parser = Parser::new(errors, &mut tokens, &bump);
        parser.parse()
    })
}
