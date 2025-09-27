use std::str::{CharIndices, SplitInclusive};

use crate::{ast::*, *};

pub struct Tokenizer<'a> {
    source: &'a [u8],
    pos: usize,
    file: SourceFile,
    errors: &'a dyn Errors,
}

fn is_whitespace(ch: u8) -> bool {
    match ch {
        b' ' | b'\t' | b'\r' => true,
        _ => false,
    }
}

fn token_spelling(tok: TokenKind) -> Option<&'static [u8]> {
    let spelling: &[u8] = match tok {
        TokenKind::Assign => b"=",
        TokenKind::Equals => b"==",
        TokenKind::BraceOpen => b"(",
        TokenKind::BraceClose => b")",
        _ => return None,
    };
    Some(spelling)
}

impl<'a> Tokenizer<'a> {
    fn new(errors: &'a dyn Errors, file: SourceFile, source: &'a [u8]) -> Tokenizer<'a> {
        Tokenizer {
            source,
            pos: 0,
            file,
            errors,
        }
    }

    fn skip_whitespace(&mut self) {
        let src = self.source;
        let mut pos = self.pos;

        loop {
            while pos < src.len() && is_whitespace(src[pos]) {
                pos += 1;
            }

            if pos + 2 <= src.len() && src[pos] == b'/' && src[pos + 1] == b'/' {
                if let Some(p) = memchr::memchr(b'\n', &src[pos..]) {
                    pos = p + 1;
                } else {
                    pos = src.len();
                }
                continue;
            }

            break;
        }

        self.pos = pos;
    }

    fn finish_ident(&mut self) -> (usize, TokenKind) {
        let src = self.source;
        let begin = self.pos;
        let mut pos = begin;
        loop {
            let cur = if pos < src.len() { src[pos] } else { b'\0' };
            if matches!(cur, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_') {
                pos += 1;
            } else {
                break;
            }
        }

        let kind = match &src[begin..pos] {
            b"class" => TokenKind::KeywordClass,
            b"struct" => TokenKind::KeywordStruct,
            _ => TokenKind::Ident,
        };

        (pos - begin, kind)
    }

    fn finish_number(&mut self) -> (usize, TokenKind) {
        todo!()
    }

    fn finish_newline(&mut self) -> (usize, TokenKind) {
        (1, TokenKind::Newline)
    }

    fn read_token(&mut self) -> Option<(usize, TokenKind)> {
        let src = self.source;
        let pos = self.pos;

        let cur = if pos < src.len() { src[pos] } else { b'\0' };
        let next = if pos + 1 < src.len() { src[pos + 1] } else { b'\0' };
        let (len, kind) = match cur {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.finish_ident(),
            b'0'..=b'9' => self.finish_number(),
            b'\n' => self.finish_newline(),
            b'=' => match next {
                b'=' => (2, TokenKind::Equals),
                _ => (1, TokenKind::Assign),
            },
            b'>' => match next {
                b'=' => (2, TokenKind::GreaterEquals),
                _ => (1, TokenKind::Greater),
            },
            b'<' => match next {
                b'=' => (2, TokenKind::LessEquals),
                _ => (1, TokenKind::Less),
            },
            b'{' => (1, TokenKind::BraceOpen),
            b'}' => (1, TokenKind::BraceClose),
            b'+' => (1, TokenKind::Add),
            _ => return None,
        };

        Some((len, kind))
    }

    fn bad_token(&mut self) -> (usize, TokenKind) {
        let loc = SourceSpan::new(self.file, self.pos, self.pos + 1);
        let ch = *self.source.get(self.pos).expect("should have returned End");
        let ch_str = match ch {
            0x20..0x7e => format!("'{}'", ch as char),
            _ => format!("(byte 0x{:02x})", ch),
        };
        error!(self, &loc, "unrecognized token: {}", ch_str);

        let begin = self.pos;
        loop {
            self.pos += 1;
            let ch = self.source.get(self.pos).map(|c| *c).unwrap_or(b'\0');
            if is_whitespace(ch) || self.read_token().is_some() {
                break;
            }
        }

        (self.pos - begin, TokenKind::Error)
    }

    fn scan(&mut self) -> Option<Token> {
        self.skip_whitespace();

        if self.pos >= self.source.len() {
            return None;
        }

        let begin = self.pos;
        let (len, kind) = match self.read_token() {
            Some(pair) => pair,
            None => self.bad_token(),
        };

        let end = begin + len;
        let span = SourceSpan::new(self.file, begin, end);

        self.pos = end;

        Some(Token { kind, span })
    }
}

impl<'a> Errors for Tokenizer<'a> {
    fn push(&self, int_loc: &InternalLocation, loc: &dyn Locatable, message: String, context: Vec<String>) {
        self.errors.push(int_loc, loc, message, context);
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.scan()
    }
}

pub fn tokenize<'a>(errors: &'a dyn Errors, file: SourceFile, source: &'a [u8]) -> Tokenizer<'a> {
    Tokenizer::new(errors, file, source)
}
