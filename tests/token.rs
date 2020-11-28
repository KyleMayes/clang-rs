use clang::*;
use clang::token::*;

pub fn test(clang: &Clang) {
    super::with_translation_unit(&clang, "test.cpp", "int a = 322; ", &[], |_, f, tu| {
        let file = tu.get_file(f).unwrap();

        let tokens = range!(file, 1, 1, 1, 13).tokenize();
        assert_eq!(tokens.len(), 5);

        macro_rules! assert_token_eq {
            ($token:expr, $kind:ident, $spelling:expr, $line:expr, $column:expr, $range:expr) => ({
                let token = $token;
                assert_eq!(token.get_kind(), TokenKind::$kind);
                assert_eq!(token.get_spelling(), $spelling);
                assert_eq!(token.get_location(), file.get_location($line, $column));
                assert_eq!(token.get_range(), $range)
            });
        }

        assert_token_eq!(tokens[0], Keyword, "int", 1, 1, range!(file, 1, 1, 1, 4));
        assert_token_eq!(tokens[1], Identifier, "a", 1, 5, range!(file, 1, 5, 1, 6));
        assert_token_eq!(tokens[2], Punctuation, "=", 1, 7, range!(file, 1, 7, 1, 8));
        assert_token_eq!(tokens[3], Literal, "322", 1, 9, range!(file, 1, 9, 1, 12));
        assert_token_eq!(tokens[4], Punctuation, ";", 1, 12, range!(file, 1, 12, 1, 13));

        fn test_annotate<'tu>(tu: &'tu TranslationUnit<'tu>, tokens: &[Token<'tu>]) {
            let declaration = tu.get_entity().get_children()[0];
            let literal = declaration.get_children()[0];
            assert_eq!(tu.annotate(tokens), &[
                Some(declaration),
                Some(declaration),
                Some(declaration),
                Some(literal),
                None,
            ]);
        }

        test_annotate(&tu, &tokens);
    });
}
