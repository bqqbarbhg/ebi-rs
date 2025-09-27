use ebi::{Compiler, SourceSpan};
use ebi::ast::{TokenKind};
use ebi::front::tokenize;

#[test]
fn tokenizer_hello() {
    let compiler = Compiler::new();
    let source = "Hello World";
    let file = compiler.add_file("internal.ebi", source.bytes().collect());

    let tokens = tokenize(&compiler, file.file(), file.data()).collect::<Vec<_>>();
    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens[0].kind, TokenKind::Ident);
    assert_eq!(tokens[1].kind, TokenKind::Ident);

    let f = file.file();
    assert_eq!(tokens[0].span, SourceSpan::new(f, 0, 5));
    assert_eq!(tokens[1].span, SourceSpan::new(f, 6, 11));
}

#[test]
fn tokenizer_error() {
    let compiler = Compiler::new();
    let source = "Hello @@@ World";
    let file = compiler.add_file("internal.ebi", source.bytes().collect());

    let tokens = tokenize(&compiler, file.file(), file.data()).collect::<Vec<_>>();
    assert_eq!(tokens.len(), 3);

    assert_eq!(tokens[0].kind, TokenKind::Ident);
    assert_eq!(tokens[1].kind, TokenKind::Error);
    assert_eq!(tokens[2].kind, TokenKind::Ident);

    let f = file.file();
    assert_eq!(tokens[0].span, SourceSpan::new(f, 0, 5));
    assert_eq!(tokens[1].span, SourceSpan::new(f, 6, 9));
    assert_eq!(tokens[2].span, SourceSpan::new(f, 10, 15));
}
