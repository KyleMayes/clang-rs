use std::ffi::{OsStr};
use std::path::{Path};

use clang::*;
use clang::source::*;

pub fn test(clang: &Clang) {
    // File ______________________________________

    super::with_file(&clang, "int a = 322;", |_, f| {
        #[cfg(feature="clang_6_0")]
        fn test_get_contents(file: &File) {
            assert_eq!(file.get_contents(), Some("int a = 322;".into()));
        }

        #[cfg(not(feature="clang_6_0"))]
        fn test_get_contents(_: &File) { }

        test_get_contents(&f);
    });

    super::with_file(&clang, "int a = 322;", |p, f| {
        assert_eq!(f.get_path(), p.to_path_buf());
        assert!(f.get_time() != 0);
        super::with_file(&clang, "int a = 322;", |_, g| assert!(f.get_id() != g.get_id()));
        assert_eq!(f.get_skipped_ranges(), &[]);
        assert!(!f.is_include_guarded());
    });

    let source = "
        #if 0
        int skipped = 32;
        #endif
        int unskipped = 32;
    ";

    super::with_temporary_file("test.cpp", source, |_, f| {
        let index = Index::new(&clang, false, false);
        let tu = index.parser(f).detailed_preprocessing_record(true).parse().unwrap();

        #[cfg(feature="clang_4_0")]
        fn test_get_skipped_ranges<'tu>(tu: TranslationUnit<'tu>, f: &Path) {
            let file = tu.get_file(f).unwrap();
            if cfg!(feature="clang_6_0") {
                assert_eq!(tu.get_skipped_ranges(), &[range!(file, 2, 9, 4, 15)]);
                assert_eq!(file.get_skipped_ranges(), &[range!(file, 2, 9, 4, 15)]);
            } else {
                assert_eq!(tu.get_skipped_ranges(), &[range!(file, 2, 10, 4, 15)]);
                assert_eq!(file.get_skipped_ranges(), &[range!(file, 2, 10, 4, 15)]);
            }
        }

        #[cfg(not(feature="clang_4_0"))]
        fn test_get_skipped_ranges<'tu>(tu: TranslationUnit<'tu>, f: &Path) {
            let file = tu.get_file(f).unwrap();
            assert_eq!(file.get_skipped_ranges(), &[range!(file, 2, 10, 4, 15)]);
        }

        test_get_skipped_ranges(tu, f);
    });

    super::with_file(&clang, "#ifndef _TEST_H_\n#define _TEST_H_\nint a = 322;\n#endif", |_, f| {
        assert!(f.is_include_guarded());
    });

    let source = r#"
        void f() {
            int a = 2 + 2;
            double b = 0.25 * 2.0;
            const char* c = "Hello, world!";
        }
    "#;

    super::with_temporary_file("test.cpp", source, |_, f| {
        let index = Index::new(&clang, false, false);
        let tu = index.parser(f).detailed_preprocessing_record(true).parse().unwrap();
        let tukids = tu.get_entity().get_children();
        let child = tukids.first().unwrap();

        // This may fail, if clang internals DO have a source
        let file = child.get_location().unwrap().get_file_location().file;
        assert_eq!(file, None);
    });


    // Module ____________________________________

    let files = &[
        ("module.modulemap", "module parent { module child [system] { header \"test.hpp\" } }"),
        ("test.hpp", ""),
        ("test.cpp", "#include \"test.hpp\""),
    ];

    super::with_temporary_files(files, |_, fs| {
        // Fails with clang 3.5 on Travis CI for some reason...
        if cfg!(feature="clang_3_6") {
            let index = Index::new(&clang, false, false);
            let tu = index.parser(&fs[2]).arguments(&["-fmodules"]).parse().unwrap();

            let module = tu.get_file(&fs[1]).unwrap().get_module().unwrap();
            assert_eq!(module.get_file().get_path().extension(), Some(OsStr::new("pcm")));
            assert_eq!(module.get_full_name(), "parent.child");
            assert_eq!(module.get_name(), "child");
            assert_eq!(module.get_top_level_headers(), &[tu.get_file(&fs[1]).unwrap()]);
            assert!(module.is_system());

            let module = module.get_parent().unwrap();
            assert_eq!(module.get_file().get_path().extension(), Some(OsStr::new("pcm")));
            assert_eq!(module.get_full_name(), "parent");
            assert_eq!(module.get_name(), "parent");
            assert_eq!(module.get_parent(), None);
            assert_eq!(module.get_top_level_headers(), &[]);
            assert!(!module.is_system());
        }
    });

    // SourceLocation ____________________________

    let source = "
        #define ADD(LEFT, RIGHT) (LEFT + RIGHT)
        #line 322 \"presumed.hpp\"
        int add(int left, int right) { return ADD(left, right); }
    ";

    super::with_file(&clang, source, |_, f| {
        let location = f.get_location(3, 51);
        assert_location_eq!(location.get_expansion_location(), Some(f), 3, 33, 81);
        assert_location_eq!(location.get_file_location(), Some(f), 3, 33, 81);
        assert_eq!(location.get_presumed_location(), ("presumed.hpp".into(), 321, 33));
        assert_location_eq!(location.get_spelling_location(), Some(f), 3, 33, 81);
        assert!(location.is_in_main_file());
        assert!(!location.is_in_system_header());
    });

    // SourceRange _______________________________

    super::with_file(&clang, "int a = 322;", |_, f| {
        let range = range!(f, 1, 5, 1, 6);
        assert_location_eq!(range.get_start().get_spelling_location(), Some(f), 1, 5, 4);
        assert_location_eq!(range.get_end().get_spelling_location(), Some(f), 1, 6, 5);
    });

}
