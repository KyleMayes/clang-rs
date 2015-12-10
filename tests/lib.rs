extern crate clang;
extern crate uuid;

use std::env;
use std::fs;
use std::io::{Write};
use std::path::{Path, PathBuf};

use clang::*;

use uuid::{Uuid};

//================================================
// Macros
//================================================

macro_rules! assert_location_eq {
    ($location:expr, $file:expr, $line:expr, $column:expr, $offset:expr) => ({
        let location = Location { file: $file, line: $line, column: $column, offset: $offset };
        assert_eq!($location, location);
    })
}

macro_rules! range {
    ($file:expr, $sl:expr, $sc:expr, $el:expr, $ec:expr) => ({
        SourceRange::new($file.get_location($sl, $sc), $file.get_location($el, $ec))
    })
}

//================================================
// Functions
//================================================

fn with_entity<'c, F: FnOnce(Entity)>(clang: &'c Clang, contents: &str, f: F) {
    with_translation_unit(clang, "test.cpp", contents, &[], |_, _, tu| f(tu.get_entity()));
}

fn with_file<'c, F: FnOnce(&Path, File)>(clang: &'c Clang, contents: &str, f: F) {
    with_translation_unit(clang, "test.cpp", contents, &[], |_, file, tu| {
        f(file, tu.get_file(file).unwrap())
    });
}

fn with_temporary_directory<F: FnOnce(&Path)>(f: F) {
    let directory = env::temp_dir().join(Uuid::new_v4().to_simple_string());
    fs::create_dir(&directory).unwrap();
    f(&directory);
    fs::remove_dir_all(&directory).unwrap();
}

fn with_temporary_file<F: FnOnce(&Path, &Path)>(name: &str, contents: &str, f: F) {
    with_temporary_files(&[(name, contents)], |d, fs| f(d, &fs[0]));
}

fn with_temporary_files<F: FnOnce(&Path, Vec<PathBuf>)>(files: &[(&str, &str)], f: F) {
    with_temporary_directory(|d| {
        let files = files.iter().map(|&(n, v)| {
            let file = d.join(n);
            fs::File::create(&file).unwrap().write_all(v.as_bytes()).unwrap();
            file
        }).collect::<Vec<_>>();

        f(d, files);
    });
}

fn with_translation_unit<'c, F>(
    clang: &'c Clang, name: &str, contents: &str, arguments: &[&str], f: F
) where F: FnOnce(&Path, &Path, TranslationUnit) {
    with_temporary_file(name, contents, |d, file| {
        let mut index = Index::new(clang, false, false);
        let options = ParseOptions::default();
        let tu = TranslationUnit::from_source(&mut index, file, arguments, &[], options).unwrap();
        f(d, &file, tu);
    });
}

//================================================
// Tests
//================================================

#[test]
fn test() {
    let clang = Clang::new().unwrap();

    // File ______________________________________

    with_file(&clang, "int a = 322;", |p, f| {
        with_file(&clang, "int a = 322;", |_, g| assert!(f.get_id() != g.get_id()));
        assert_eq!(f.get_path(), p.to_path_buf());
        assert!(f.get_time() != 0);
        assert!(!f.is_include_guarded());
    });

    with_file(&clang, "#ifndef _TEST_H_\n#define _TEST_H_\nint a = 322;\n#endif", |_, f| {
        assert!(f.is_include_guarded());
    });

    // Diagnostic ________________________________

    let source = "
        int add(float a, float b) { return a + b; }
        template <typename T> struct A { typedef T::U dependent; };
        struct Integer { int i; }; Integer i = { i: 0 };
    ";

    with_translation_unit(&clang, "test.cpp", source, &["-Wconversion"], |_, f, tu| {
        let mut options = FormatOptions::default();
        options.display_source_location = false;
        options.display_option = false;

        let file = tu.get_file(f).unwrap();

        let diagnostics = tu.get_diagnostics();
        assert_eq!(diagnostics.len(), 3);

        let text = "implicit conversion turns floating-point number into integer: 'float' to 'int'";
        assert_eq!(diagnostics[0].format(options), format!("warning: {}", text));
        assert_eq!(diagnostics[0].get_fix_its(), &[]);
        assert_eq!(diagnostics[0].get_location(), file.get_location(2, 46));
        assert_eq!(diagnostics[0].get_ranges(), &[
            range!(file, 2, 44, 2, 49),
            range!(file, 2, 37, 2, 43),
        ]);
        assert_eq!(diagnostics[0].get_severity(), Severity::Warning);
        assert_eq!(diagnostics[0].get_text(), text);

        let text = "missing 'typename' prior to dependent type name 'T::U'";
        assert_eq!(diagnostics[1].format(options), format!("error: {}", text));
        assert_eq!(diagnostics[1].get_fix_its(), &[
            FixIt::Insertion(file.get_location(3, 50), "typename ".into())
        ]);
        assert_eq!(diagnostics[1].get_location(), file.get_location(3, 50));
        assert_eq!(diagnostics[1].get_ranges(), &[range!(file, 3, 50, 3, 54)]);
        assert_eq!(diagnostics[1].get_severity(), Severity::Error);
        assert_eq!(diagnostics[1].get_text(), text);

        let text = "use of GNU old-style field designator extension";
        assert_eq!(diagnostics[2].format(options), format!("warning: {}", text));
        let range = range!(file, 4, 50, 4, 52);
        assert_eq!(diagnostics[2].get_fix_its(), &[FixIt::Replacement(range, ".i = ".into())]);
        assert_eq!(diagnostics[2].get_location(), range.get_start());
        assert_eq!(diagnostics[2].get_ranges(), &[]);
        assert_eq!(diagnostics[2].get_severity(), Severity::Warning);
        assert_eq!(diagnostics[2].get_text(), text);
    });

    // Entity ____________________________________

    with_translation_unit(&clang, "test.cpp", "int a = 322;", &[], |_, f, tu| {
        let file = tu.get_file(f).unwrap();

        let entity = tu.get_entity();
        assert_eq!(entity.get_display_name(), Some(f.to_str().unwrap().into()));
        assert_eq!(entity.get_kind(), EntityKind::TranslationUnit);
        assert_eq!(entity.get_location(), None);
        assert_eq!(entity.get_mangled_name(), None);
        assert_eq!(entity.get_name(), Some(f.to_str().unwrap().into()));
        assert_eq!(entity.get_name_ranges(), &[]);
        assert_eq!(entity.get_translation_unit().get_file(f), tu.get_file(f));

        let children = entity.get_children();
        assert_eq!(children.len(), 1);

        assert_eq!(children[0].get_display_name(), Some("a".into()));
        assert_eq!(children[0].get_kind(), EntityKind::VarDecl);
        assert_eq!(children[0].get_location(), Some(file.get_location(1, 5)));
        assert_eq!(children[0].get_mangled_name(), Some("_Z1a".into()));
        assert_eq!(children[0].get_name(), Some("a".into()));
        assert_eq!(children[0].get_name_ranges(), &[range!(file, 1, 5, 1, 6)]);
        assert_eq!(children[0].get_range(), Some(range!(file, 1, 1, 1, 12)));
        assert_eq!(children[0].get_translation_unit().get_file(f), tu.get_file(f));
    });

    let source = "
        class B { };
        class A : public B {
        private:
            void a() { };
        protected:
            void b() { };
        public:
            void c() { };
        };
    ";

    with_entity(&clang, source, |e| {
        assert_eq!(e.get_accessibility(), None);

        let children = e.get_children()[1].get_children();
        assert_eq!(children.len(), 7);

        assert_eq!(children[0].get_accessibility(), Some(Accessibility::Public));
        assert_eq!(children[1].get_accessibility(), Some(Accessibility::Private));
        assert_eq!(children[2].get_accessibility(), Some(Accessibility::Private));
        assert_eq!(children[3].get_accessibility(), Some(Accessibility::Protected));
        assert_eq!(children[4].get_accessibility(), Some(Accessibility::Protected));
        assert_eq!(children[5].get_accessibility(), Some(Accessibility::Public));
        assert_eq!(children[6].get_accessibility(), Some(Accessibility::Public));
    });

    let source = "
        struct A;
        struct A;
        struct A { int a; };
    ";

    with_entity(&clang, source, |e| {
        let children = e.get_children();
        assert_eq!(children.len(), 3);

        assert_eq!(children[0].get_canonical_entity(), children[0]);
        assert_eq!(children[0].get_definition(), Some(children[2]));

        assert_eq!(children[1].get_canonical_entity(), children[0]);
        assert_eq!(children[1].get_definition(), Some(children[2]));

        assert_eq!(children[2].get_canonical_entity(), children[0]);
        assert_eq!(children[2].get_definition(), Some(children[2]));
    });

    let source = "
        int a;
        /// \\brief A global integer.
        int b;
    ";

    with_translation_unit(&clang, "test.cpp", source, &[], |_, f, tu| {
        let file = tu.get_file(f).unwrap();

        let children = tu.get_entity().get_children();
        assert_eq!(children.len(), 2);

        assert_eq!(file.get_location(2, 13).get_entity(), Some(children[0]));
        assert_eq!(file.get_location(3, 13).get_entity(), None);
        assert_eq!(file.get_location(4, 13).get_entity(), Some(children[1]));

        assert_eq!(children[0].get_comment(), None);
        assert_eq!(children[0].get_comment_brief(), None);
        assert_eq!(children[0].get_comment_range(), None);

        assert_eq!(children[1].get_comment(), Some("/// \\brief A global integer.".into()));
        assert_eq!(children[1].get_comment_brief(), Some("A global integer.".into()));
        assert_eq!(children[1].get_comment_range(), Some(range!(file, 3, 9, 3, 39)));
    });

    let source = "
        struct A { struct { int b; }; int i : 322; };
    ";

    with_entity(&clang, source, |e| {
        let children = e.get_children();
        assert_eq!(children.len(), 1);

        assert!(!children[0].is_anonymous());

        let children = children[0].get_children();
        assert_eq!(children.len(), 2);

        assert_eq!(children[0].get_name(), None);
        assert_eq!(children[0].get_display_name(), None);
        assert!(children[0].is_anonymous());
        assert!(!children[0].is_bit_field());

        assert_eq!(children[1].get_name(), Some("i".into()));
        assert_eq!(children[1].get_display_name(), Some("i".into()));
        assert!(!children[1].is_anonymous());
        assert!(children[1].is_bit_field());
    });

    let source = "
        void a() { }
        class B { void b() { } };
    ";

    with_entity(&clang, source, |e| {
        assert_eq!(e.get_language(), None);

        let children = e.get_children();
        assert_eq!(children.len(), 2);

        assert_eq!(children[0].get_language(), Some(Language::C));
        assert_eq!(children[1].get_language(), Some(Language::Cpp));
    });

    let source = "
        struct A { void a(); };
        void A::a() { }
    ";

    with_entity(&clang, source, |e| {
        assert_eq!(e.get_lexical_parent(), None);
        assert_eq!(e.get_semantic_parent(), None);

        let children = e.get_children();
        assert_eq!(children.len(), 2);

        assert_eq!(children[0].get_lexical_parent(), Some(e));
        assert_eq!(children[0].get_semantic_parent(), Some(e));

        assert_eq!(children[1].get_lexical_parent(), Some(e));
        assert_eq!(children[1].get_semantic_parent(), Some(children[0]));
    });

    let source = "
        class Class {
            void a() const { }
            virtual void b() = 0;
            static void c() { }
            virtual void d() { }
        };
    ";

    with_entity(&clang, source, |e| {
        let children = e.get_children()[0].get_children();
        assert_eq!(children.len(), 4);

        macro_rules! method {
            ($entity:expr, $c:expr, $pv:expr, $s:expr, $v:expr) => ({
                assert_eq!($entity.is_const_method(), $c);
                assert_eq!($entity.is_pure_virtual_method(), $pv);
                assert_eq!($entity.is_static_method(), $s);
                assert_eq!($entity.is_virtual_method(), $v);
            });
        }

        method!(children[0], true, false, false, false);
        method!(children[1], false, true, false, true);
        method!(children[2], false, false, true, false);
        method!(children[3], false, false, false, true);
    });

    let source = "
        struct A {
            void a() { }
            virtual void b() { }
        };

        void function() {
            A a;
            a.a();
            a.b();
        }
    ";

    with_entity(&clang, source, |e| {
        let children = e.get_children()[1].get_children()[0].get_children();
        assert_eq!(children.len(), 3);

        assert!(!children[1].is_dynamic_call());
        assert!(children[2].is_dynamic_call());
    });

    let source = "
        void a() { }
        void b(...) { }
    ";

    with_entity(&clang, source, |e| {
        let children = e.get_children();
        assert_eq!(children.len(), 2);

        assert_eq!(children[0].get_display_name(), Some("a()".into()));
        assert!(!children[0].is_variadic());

        assert_eq!(children[1].get_display_name(), Some("b(...)".into()));
        assert!(children[1].is_variadic());
    });

    // Index _____________________________________

    let mut index = Index::new(&clang, false, false);

    let mut options = ThreadOptions::default();
    assert_eq!(index.get_thread_options(), options);

    options.editing = true;
    index.set_thread_options(options);
    assert_eq!(index.get_thread_options(), options);

    // Module ____________________________________

    let files = &[
        ("module.modulemap", "module parent { module child [system] { header \"test.hpp\" } }"),
        ("test.hpp", ""),
        ("test.cpp", "#include \"test.hpp\""),
    ];

    with_temporary_files(files, |_, fs| {
        let mut index = Index::new(&clang, false, false);
        let arguments = &["-fmodules"];
        let options = ParseOptions::default();
        let tu = TranslationUnit::from_source(&mut index, &fs[2], arguments, &[], options).unwrap();

        let module = tu.get_file(&fs[1]).unwrap().get_module().unwrap();
        assert_eq!(module.get_file().get_path().extension(), Some(std::ffi::OsStr::new("pcm")));
        assert_eq!(module.get_full_name(), "parent.child");
        assert_eq!(module.get_name(), "child");
        assert_eq!(module.get_top_level_headers(), &[tu.get_file(&fs[1]).unwrap()]);
        assert!(module.is_system());

        let module = module.get_parent().unwrap();
        assert_eq!(module.get_file().get_path().extension(), Some(std::ffi::OsStr::new("pcm")));
        assert_eq!(module.get_full_name(), "parent");
        assert_eq!(module.get_name(), "parent");
        assert_eq!(module.get_parent(), None);
        assert_eq!(module.get_top_level_headers(), &[]);
        assert!(!module.is_system());
    });

    // SourceLocation ____________________________

    let source = "
        #define ADD(LEFT, RIGHT) (LEFT + RIGHT)
        #line 322 \"presumed.hpp\"
        int add(int left, int right) { return ADD(left, right); }
    ";

    with_file(&clang, source, |_, f| {
        let location = f.get_location(3, 51);
        assert_location_eq!(location.get_expansion_location(), f, 3, 33, 81);
        assert_location_eq!(location.get_file_location(), f, 3, 33, 81);
        assert_eq!(location.get_presumed_location(), ("presumed.hpp".into(), 321, 33));
        assert_location_eq!(location.get_spelling_location(), f, 3, 33, 81);
        assert!(location.is_in_main_file());
        assert!(!location.is_in_system_header());
    });

    // SourceRange _______________________________

    with_file(&clang, "int a = 322", |_, f| {
        let range = range!(f, 1, 5, 1, 6);
        assert_location_eq!(range.get_end().get_spelling_location(), f, 1, 6, 5);
        assert_location_eq!(range.get_start().get_spelling_location(), f, 1, 5, 4);
    });

    // TranslationUnit ___________________________

    //- from_ast ---------------------------------
    //- save -------------------------------------

    with_translation_unit(&clang, "test.cpp", "int a = 322;", &[], |d, _, tu| {
        let file = d.join("test.cpp.gch");
        tu.save(&file).unwrap();
        let mut index = Index::new(&clang, false, false);
        let _ = TranslationUnit::from_ast(&mut index, &file).unwrap();
    });

    //- from_source ------------------------------

    with_temporary_file("test.cpp", "int a = 322;", |_, f| {
        let mut index = Index::new(&clang, false, false);
        let unsaved = &[Unsaved::new(f, "int a = 644;")];
        let options = ParseOptions::default();
        let _ = TranslationUnit::from_source(&mut index, f, &[], unsaved, options).unwrap();
    });

    //- get_file ---------------------------------

    with_translation_unit(&clang, "test.cpp", "int a = 322;", &[], |d, _, tu| {
        assert_eq!(tu.get_file(d.join("test.c")), None);
    });

    //- get_memory_usage -------------------------

    with_translation_unit(&clang, "test.cpp", "int a = 322;", &[], |_, _, tu| {
        let usage = tu.get_memory_usage();
        assert_eq!(usage.get(&MemoryUsage::Selectors), Some(&0));
    });

    //- reparse ----------------------------------

    with_translation_unit(&clang, "test.cpp", "int a = 322;", &[], |_, f, tu| {
        let _ = tu.reparse(&[Unsaved::new(f, "int a = 644;")]).unwrap();
    });
}
