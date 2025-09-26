use ebi::{front::{tokenize, TokenKind}, Compiler, SourceSpan};


#[test]
fn tokenizer_hello() {
    let compiler = Compiler::new();
    let file = compiler.get_file("internal.ebi");
    let source = "Hello World";

    let tokens = tokenize(&compiler, file, source.as_bytes());
    assert_eq!(tokens.len(), 3);

    assert_eq!(tokens[0].kind(), TokenKind::Ident);
    assert_eq!(tokens[1].kind(), TokenKind::Ident);
    assert_eq!(tokens[2].kind(), TokenKind::End);

    assert_eq!(tokens[0].span(), SourceSpan::new(file, 0, 5));
    assert_eq!(tokens[1].span(), SourceSpan::new(file, 6, 11));
    assert_eq!(tokens[2].span(), SourceSpan::new(file, 11, 11));
}
