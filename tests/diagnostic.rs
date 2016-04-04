use clang::*;
use clang::diagnostic::*;

pub fn test(clang: &Clang) {
    let source = "
        int add(float a, float b) { return a + b; }
        template <typename T> struct A { typedef T::U dependent; };
        struct Integer { int i; }; Integer i = { i: 0 };
    ";

    super::with_translation_unit(&clang, "test.cpp", source, &["-Wconversion"], |_, f, tu| {
        let file = tu.get_file(f).unwrap();

        let diagnostics = tu.get_diagnostics();
        assert_eq!(diagnostics.len(), 3);

        macro_rules! assert_diagnostic_eq {
            ($diagnostic:expr, $severity:expr, $text:expr, $location:expr, $ranges:expr, $fix_its:expr) => ({
                let diagnostic = $diagnostic;
                assert_eq!(diagnostic.get_severity(), $severity);
                assert_eq!(diagnostic.get_text(), $text);
                assert_eq!(diagnostic.get_location(), $location);
                assert_eq!(diagnostic.get_ranges(), $ranges);
                assert_eq!(diagnostic.get_fix_its(), $fix_its);
                assert!(diagnostic.get_children().is_empty());
                let actual = diagnostic.formatter().source_location(false).option(false).format();
                let expected = match $severity {
                    Severity::Warning => format!("warning: {}", $text),
                    Severity::Error => format!("error: {}", $text),
                    _ => unreachable!(),
                };
                assert_eq!(actual, expected);
            });
        }

        let text = "implicit conversion turns floating-point number into integer: 'float' to 'int'";
        assert_diagnostic_eq!(diagnostics[0], Severity::Warning, text, file.get_location(2, 46), &[
            range!(file, 2, 44, 2, 49),
            range!(file, 2, 37, 2, 43),
        ], &[
        ]);

        let text = "missing 'typename' prior to dependent type name 'T::U'";
        assert_diagnostic_eq!(diagnostics[1], Severity::Error, text, file.get_location(3, 50), &[
            range!(file, 3, 50, 3, 54)
        ], &[
            FixIt::Insertion(file.get_location(3, 50), "typename ".into())
        ]);

        let text = "use of GNU old-style field designator extension";
        assert_diagnostic_eq!(diagnostics[2], Severity::Warning, text, file.get_location(4, 50), &[
        ], &[
            FixIt::Replacement(range!(file, 4, 50, 4, 52), ".i = ".into())
        ]);
    });
}
