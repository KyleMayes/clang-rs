extern crate clang;
extern crate libc;

use std::env;
use std::fs;
use std::mem;
use std::io::{Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use clang::*;
use clang::completion::*;
use clang::source::*;

use libc::{c_int};

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
        let start = $file.get_location($sl, $sc);
        ::clang::source::SourceRange::new(start, $file.get_location($el, $ec))
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

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn with_temporary_directory<F: FnOnce(&Path)>(f: F) {
    let exe = env::current_exe().unwrap().file_name().unwrap().to_string_lossy().into_owned();
    let mut path;

    loop {
        path = env::temp_dir().join(format!("{}{}", exe, COUNTER.fetch_add(1, Ordering::SeqCst)));

        if !path.exists() {
            break;
        }
    }

    fs::create_dir(&path).unwrap();
    f(&path);
    fs::remove_dir_all(&path).unwrap();
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
        let index = Index::new(clang, false, false);
        f(d, &file, index.parser(file).arguments(arguments).parse().unwrap());
    });
}

fn with_types<'c, F: FnOnce(Vec<Type>)>(clang: &'c Clang, contents: &str, f: F) {
    with_translation_unit(clang, "test.cpp", contents, &[], |_, _, tu| {
        f(tu.get_entity().get_children().iter().flat_map(|e| e.get_type().into_iter()).collect());
    });
}

//================================================
// Tests
//================================================

#[path="completion.rs"]
mod completion_test;
#[path="diagnostic.rs"]
mod diagnostic_test;
#[path="documentation.rs"]
mod documentation_test;
#[path="source.rs"]
mod source_test;
#[path="token.rs"]
mod token_test;

#[path="sonar.rs"]
mod sonar_test;

#[test]
fn test() {
    let clang = Clang::new().unwrap();

    println!("libclang: {}", get_version());

    completion_test::test(&clang);
    diagnostic_test::test(&clang);
    documentation_test::test(&clang);
    source_test::test(&clang);
    token_test::test(&clang);

    sonar_test::test(&clang);

    // SourceError _______________________________

    assert_eq!(format!("{}", SourceError::Unknown), "an unknown error occurred");

    // Entity ____________________________________

    let source = "
        struct B { };
    ";

    with_entity(&clang, source, |e| {
        #[cfg(feature="clang_6_0")]
        fn test_is_abstract_record(children: &[Entity]) {
            assert_eq!(children.len(), 1);
            assert!(!children[0].is_abstract_record());
        }

        #[cfg(not(feature="clang_6_0"))]
        fn test_is_abstract_record(_: &[Entity]) { }

        test_is_abstract_record(&e.get_children()[..]);
    });

    let source = "
        thread_local int foo;
        int bar;
    ";

    with_entity(&clang, source, |e| {
        #[cfg(feature="clang_6_0")]
        fn test_get_tls_kind(children: &[Entity]) {
            assert_eq!(children.len(), 2);
            assert_eq!(children[0].get_tls_kind(), Some(TlsKind::Dynamic));
            assert_eq!(children[1].get_tls_kind(), None);
        }

        #[cfg(not(feature="clang_6_0"))]
        fn test_get_tls_kind(_: &[Entity]) { }

        test_get_tls_kind(&e.get_children()[..]);
    });

    with_translation_unit(&clang, "test.cpp", "int a = 322;", &[], |_, f, tu| {
        #[cfg(feature="clang_5_0")]
        fn test_target(tu: &TranslationUnit) {
            let target = tu.get_target();
            assert!(!target.triple.is_empty());
            assert_eq!(target.pointer_width, mem::size_of::<usize>() * 8);
        }

        #[cfg(not(feature="clang_5_0"))]
        fn test_target(_: &TranslationUnit) { }

        let file = tu.get_file(f).unwrap();

        let entity = tu.get_entity();
        assert_eq!(entity.get_completion_string(), None);
        assert_eq!(entity.get_display_name(), Some(f.to_str().unwrap().into()));
        assert_eq!(entity.get_kind(), EntityKind::TranslationUnit);
        assert_eq!(entity.get_location(), None);
        assert_eq!(entity.get_name(), Some(f.to_str().unwrap().into()));
        assert_eq!(entity.get_name_ranges(), &[]);
        assert_eq!(entity.get_platform_availability(), None);
        assert_eq!(entity.get_translation_unit().get_file(f), tu.get_file(f));
        assert_eq!(entity.get_usr(), None);

        let children = entity.get_children();
        assert_eq!(children.len(), 1);

        assert_eq!(children[0].get_display_name(), Some("a".into()));
        assert_eq!(children[0].get_kind(), EntityKind::VarDecl);
        assert_eq!(children[0].get_location(), Some(file.get_location(1, 5)));
        assert_eq!(children[0].get_name(), Some("a".into()));
        assert_eq!(children[0].get_name_ranges(), &[range!(file, 1, 5, 1, 6)]);
        assert_eq!(children[0].get_range(), Some(range!(file, 1, 1, 1, 12)));
        assert_eq!(children[0].get_translation_unit().get_file(f), tu.get_file(f));
        assert_eq!(children[0].get_platform_availability(), Some(vec![]));
        assert_eq!(children[0].get_usr(), Some(Usr("c:@a".into())));

        let string = children[0].get_completion_string().unwrap();
        assert_eq!(string.get_chunks(), &[
            CompletionChunk::ResultType("int".into()),
            CompletionChunk::TypedText("a".into()),
        ]);

        test_target(&tu);
    });

    let source = r#"
        void f() {
            unsigned int a = 2 + 2;
            double b = 0.25 * 2.0;
            const char* c = "Hello, world!";
        }
    "#;

    with_entity(&clang, source, |e| {
        #[cfg(feature="clang_4_0")]
        fn test_evaluate<'tu>(expressions: &[Entity<'tu>]) {
            assert_eq!(expressions[0].evaluate(), Some(EvaluationResult::UnsignedInteger(4)));
            assert_eq!(expressions[1].evaluate(), Some(EvaluationResult::Float(0.5)));
            match expressions[2].evaluate() {
                Some(EvaluationResult::String(string)) => {
                    assert_eq!(string.to_str(), Ok("Hello, world!"));
                },
                _ => unreachable!(),
            }
        }

        #[cfg(all(feature="clang_3_9", not(feature="clang_4_0")))]
        fn test_evaluate<'tu>(expressions: &[Entity<'tu>]) {
            assert_eq!(expressions[0].evaluate(), Some(EvaluationResult::SignedInteger(4)));
            assert_eq!(expressions[1].evaluate(), Some(EvaluationResult::Float(0.5)));
            match expressions[2].evaluate() {
                Some(EvaluationResult::String(string)) => {
                    assert_eq!(string.to_str(), Ok("Hello, world!"));
                },
                _ => unreachable!(),
            }
        }

        #[cfg(not(feature="clang_3_9"))]
        fn test_evaluate<'tu>(_: &[Entity<'tu>]) { }

        let children = e.get_children()[0].get_children()[0].get_children();
        let expressions = children.into_iter().map(|e| {
            e.get_children()[0].get_children()[0]
        }).collect::<Vec<_>>();
        assert_eq!(expressions.len(), 3);

        test_evaluate(&expressions);
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
        assert!(!children[0].is_definition());

        assert_eq!(children[1].get_canonical_entity(), children[0]);
        assert_eq!(children[1].get_definition(), Some(children[2]));
        assert!(!children[1].is_definition());

        assert_eq!(children[2].get_canonical_entity(), children[0]);
        assert_eq!(children[2].get_definition(), Some(children[2]));
        assert!(children[2].is_definition());
    });

    let source = "
        struct A { struct { int b; }; int i : 322; };
    ";

    with_entity(&clang, source, |e| {
        #[cfg(feature="clang_3_7")]
        fn test_is_anonymous<'tu>(children: &[Entity<'tu>]) {
            assert!(!children[0].is_anonymous());

            let children = children[0].get_children();
            assert!(children[0].is_anonymous());
            assert!(!children[1].is_anonymous());
        }

        #[cfg(not(feature="clang_3_7"))]
        fn test_is_anonymous<'tu>(_: &[Entity<'tu>]) { }

        let children = e.get_children();
        assert_eq!(children.len(), 1);

        test_is_anonymous(&children);

        let children = children[0].get_children();
        assert_eq!(children.len(), 2);

        assert_eq!(children[0].get_bit_field_width(), None);
        assert_eq!(children[0].get_name(), None);
        assert_eq!(children[0].get_display_name(), None);
        assert!(!children[0].is_bit_field());

        if !cfg!(target_os="windows") {
            assert_eq!(children[1].get_bit_field_width(), Some(322));
            assert_eq!(children[1].get_name(), Some("i".into()));
            assert_eq!(children[1].get_display_name(), Some("i".into()));
            assert!(children[1].is_bit_field());
        }
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
        unsigned int integer = 322;
        enum A { B = 322, C = 644 };
    ";

    with_entity(&clang, source, |e| {
        assert_eq!(e.get_language(), None);

        let children = e.get_children();
        assert_eq!(children.len(), 2);

        assert_eq!(children[0].get_enum_constant_value(), None);
        assert_eq!(children[0].get_enum_underlying_type(), None);

        if !cfg!(target_os="windows") {
            assert_eq!(children[1].get_enum_constant_value(), None);
            assert_eq!(children[1].get_enum_underlying_type(), Some(children[0].get_type().unwrap()));
        }

        let children = children[1].get_children();
        assert_eq!(children.len(), 2);

        assert_eq!(children[0].get_enum_constant_value(), Some((322, 322)));
        assert_eq!(children[1].get_enum_constant_value(), Some((644, 644)));
    });

    let source = "
        void a();
        void b() throw();
        void c() throw(int);
    ";

    with_entity(&clang, source, |e| {
        #[cfg(feature="clang_5_0")]
        fn test_get_exception_specification(children: &[Entity]) {
            assert_eq!(children[0].get_exception_specification(), None);
            assert_eq!(children[1].get_exception_specification(), Some(ExceptionSpecification::DynamicNone));
            assert_eq!(children[2].get_exception_specification(), Some(ExceptionSpecification::Dynamic));
        }

        #[cfg(not(feature="clang_5_0"))]
        fn test_get_exception_specification(_: &[Entity]) { }

        let children = e.get_children();
        assert_eq!(children.len(), 3);

        test_get_exception_specification(&children[..]);
    });

    let files = &[
        ("test.hpp", ""),
        ("test.cpp", "#include \"test.hpp\""),
    ];

    with_temporary_files(files, |_, fs| {
        let index = Index::new(&clang, false, false);
        let tu = index.parser(&fs[1]).detailed_preprocessing_record(true).parse().unwrap();

        let last = tu.get_entity().get_children().iter().last().unwrap().clone();
        assert_eq!(last.get_kind(), EntityKind::InclusionDirective);
        assert_eq!(last.get_file(), tu.get_file(&fs[0]));

        assert_eq!(tu.get_file(&fs[1]).unwrap().get_includes(), &[last]);
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
        void a() { }
        static void b() { }
    ";

    with_entity(&clang, source, |e| {
        #[cfg(feature="clang_3_6")]
        fn test_get_storage_class<'tu>(entity: Entity<'tu>) {
            assert_eq!(entity.get_storage_class(), None);

            let children = entity.get_children();
            assert_eq!(children[0].get_storage_class(), Some(StorageClass::None));
            assert_eq!(children[1].get_storage_class(), Some(StorageClass::Static));
        }

        #[cfg(not(feature="clang_3_6"))]
        fn test_get_storage_class<'tu>(_: Entity<'tu>) { }

        assert_eq!(e.get_linkage(), None);

        let children = e.get_children();
        assert_eq!(children.len(), 2);

        assert_eq!(children[0].get_linkage(), Some(Linkage::External));
        assert_eq!(children[1].get_linkage(), Some(Linkage::Internal));

        test_get_storage_class(e);
    });

    let source = "
        void a(int i) { }
        void a(float f) { }
        template <typename T> void b(T t) { a(t); }
    ";

    with_entity(&clang, source, |e| {
        if !cfg!(target_os="windows") {
            let children = e.get_children();
            assert_eq!(children.len(), 3);

            let children = children[2].get_children();
            assert_eq!(children.len(), 3);

            let children = children[2].get_children();
            assert_eq!(children.len(), 1);

            let children = children[0].get_children();
            assert_eq!(children.len(), 2);

            let children = children[0].get_children();
            assert_eq!(children.len(), 1);

            let declarations = vec![e.get_children()[1], e.get_children()[0]];
            assert_eq!(children[0].get_overloaded_declarations(), Some(declarations));
        }
    });

    let source = "
        struct A { virtual void a() { } };
        struct B : public A { virtual void a() { } };
    ";

    with_entity(&clang, source, |e| {
        let children = e.get_children();
        assert_eq!(children.len(), 2);

        assert_eq!(children[0].get_children()[0].get_overridden_methods(), None);
        assert_eq!(children[1].get_children()[1].get_overridden_methods(), Some(vec![
            children[0].get_children()[0]
        ]));
    });

    let source = "
        int integer = 322;
        template <typename T, int I> void function() { }
        template <> void function<int, 322>() { }
    ";

    with_entity(&clang, source, |e| {
        #[cfg(feature="clang_3_6")]
        fn test_get_template_arguments<'tu>(children: &[Entity<'tu>]) {
            assert_eq!(children[0].get_template_arguments(), None);
            assert_eq!(children[1].get_template_arguments(), None);
            assert_eq!(children[2].get_template_arguments(), Some(vec![
                TemplateArgument::Type(children[0].get_type().unwrap()),
                TemplateArgument::Integral(322, 322),
            ]));
        }

        #[cfg(not(feature="clang_3_6"))]
        fn test_get_template_arguments<'tu>(_: &[Entity<'tu>]) { }

        let children = e.get_children();
        assert_eq!(children.len(), 3);

        assert_eq!(children[0].get_template(), None);
        assert_eq!(children[0].get_template_kind(), None);

        assert_eq!(children[1].get_template(), None);
        assert_eq!(children[1].get_template_kind(), Some(EntityKind::FunctionDecl));

        assert_eq!(children[2].get_template(), Some(children[1]));
        assert_eq!(children[2].get_template_kind(), None);

        test_get_template_arguments(&children);
    });

    let source = "
        int integer = 322;
        typedef int Integer;
    ";

    with_entity(&clang, source, |e| {
        let children = e.get_children();
        assert_eq!(children.len(), 2);

        assert_eq!(children[0].get_typedef_underlying_type(), None);
        assert_eq!(children[1].get_typedef_underlying_type(), Some(children[0].get_type().unwrap()));
    });

    let source = r#"
        class A { };
        class __attribute__((visibility("hidden"))) B { };
    "#;

    with_entity(&clang, source, |e| {
        #[cfg(feature="clang_3_8")]
        fn test_get_visibility<'tu>(children: &[Entity<'tu>]) {
            assert_eq!(children[0].get_visibility(), Some(Visibility::Default));
            assert_eq!(children[1].get_visibility(), Some(Visibility::Hidden));
        }

        #[cfg(not(feature="clang_3_8"))]
        fn test_get_visibility<'tu>(_: &[Entity<'tu>]) { }

        let children = e.get_children();
        assert_eq!(children.len(), 2);

        test_get_visibility(&children);
    });

    let source = "
        void a();
        [[noreturn]] void b();
    ";

    with_translation_unit(&clang, "test.cpp", source, &["--std=c++0x"], |_, _, tu| {
        #[cfg(feature="clang_3_9")]
        fn test_attributes<'tu>(children: &[Entity<'tu>]) {
            assert!(!children[0].has_attributes());
            assert!(children[1].has_attributes());
        }

        #[cfg(not(feature="clang_3_9"))]
        fn test_attributes<'tu>(_: &[Entity<'tu>]) { }

        let children = tu.get_entity().get_children();
        assert_eq!(children.len(), 2);

        test_attributes(&children);
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
        enum A { A_A, A_B, A_C };
        enum class B { B_A, B_B, B_C };
    ";

    with_entity(&clang, source, |e| {
        #[cfg(feature="clang_5_0")]
        fn test_is_scoped(children: &[Entity]) {
            assert!(!children[0].is_scoped());
            assert!(children[1].is_scoped());
        }

        #[cfg(not(feature="clang_5_0"))]
        fn test_is_scoped(_: &[Entity]) { }

        let children = e.get_children();
        assert_eq!(children.len(), 2);

        test_is_scoped(&children[..]);
    });

    let source = "
        class Class {
            Class(int) { }
            explicit Class(const Class&) = default;
            Class() { }
            explicit Class(Class&&) = default;
        };
    ";

    with_entity(&clang, source, |e| {
        #[cfg(feature="clang_3_9")]
        fn test_constructors<'tu>(children: &[Entity<'tu>]) {
            macro_rules! constructor {
                ($entity:expr, $conv:expr, $cpy:expr, $def:expr, $defed:expr, $mov:expr) => ({
                    assert_eq!($entity.is_converting_constructor(), $conv);
                    assert_eq!($entity.is_copy_constructor(), $cpy);
                    assert_eq!($entity.is_default_constructor(), $def);
                    assert_eq!($entity.is_defaulted(), $defed);
                    assert_eq!($entity.is_move_constructor(), $mov);
                });
            }

            constructor!(children[0], true, false, false, false, false);
            constructor!(children[1], false, true, false, true, false);
            constructor!(children[2], false, false, true, false, false);
            constructor!(children[3], false, false, false, true, true);
        }

        #[cfg(not(feature="clang_3_9"))]
        fn test_constructors<'tu>(_: &[Entity<'tu>]) { }

        let children = e.get_children()[0].get_children();
        assert_eq!(children.len(), 4);

        test_constructors(&children);
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

    let source = r#"
        void a();
        void b()
            __attribute__((external_source_symbol(
                language="Swift",
                defined_in="module",
                generated_declaration)));
    "#;

    with_entity(&clang, source, |e| {
        #[cfg(feature="clang_5_0")]
        fn test_get_external_symbol(children: &[Entity]) {
            assert_eq!(children[0].get_external_symbol(), None);
            let symbol = children[1].get_external_symbol();
            assert!(symbol.is_some());
            let symbol = symbol.unwrap();
            assert_eq!(symbol.language, "Swift");
            assert_eq!(symbol.defined, "module");
            assert!(symbol.generated);
        }

        #[cfg(not(feature="clang_5_0"))]
        fn test_get_external_symbol(_: &[Entity]) { }

        let children = e.get_children();
        assert_eq!(children.len(), 2);

        test_get_external_symbol(&children[..]);
    });

    let source = "
        class A {
            int a;
            mutable int b;
        };
    ";

    with_entity(&clang, source, |e| {
        #[cfg(feature="clang_3_8")]
        fn test_is_mutable<'tu>(children: &[Entity<'tu>]) {
            assert!(!children[0].is_mutable());
            assert!(children[1].is_mutable());
        }

        #[cfg(not(feature="clang_3_8"))]
        fn test_is_mutable<'tu>(_: &[Entity<'tu>]) { }

        let children = e.get_children()[0].get_children();
        assert_eq!(children.len(), 2);

        test_is_mutable(&children);
    });

    let source = "
        void a() { }
        void b(...) { }
    ";

    with_entity(&clang, source, |e| {
        let children = e.get_children();
        assert_eq!(children.len(), 2);

        assert!(!children[0].is_variadic());
        assert!(children[1].is_variadic());
    });

    let source = "
        struct A { };
        struct B : A { };
        struct C : virtual A { };
    ";

    with_entity(&clang, source, |e| {
        let children = e.get_children();
        assert_eq!(children.len(), 3);

        assert!(!children[1].get_children()[0].is_virtual_base());
        assert!(children[2].get_children()[0].is_virtual_base());
    });

    let source = "
        @interface A
        - (int)foo;
        @end
    ";

    with_translation_unit(&clang, "test.mm", source, &[], |_, _, tu| {
        let entities = tu.get_entity().get_children();
        assert_eq!(entities.len(), 2);
        // The Protocol class reference is automatically added by clang at the start of any Objective-C file
        assert_eq!(entities[0].get_kind(), EntityKind::ObjCClassRef);
        assert_eq!(entities[0].get_name(), Some("Protocol".into()));
        assert_eq!(entities[1].get_kind(), EntityKind::ObjCInterfaceDecl);
        assert_eq!(entities[1].get_name(), Some("A".into()));

        let children = entities[1].get_children();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].get_kind(), EntityKind::ObjCInstanceMethodDecl,);
        assert_eq!(children[0].get_name(), Some("foo".into()));

        match children[0].get_result_type() {
            Some(result_type) => {
                assert_eq!(result_type.get_kind(), TypeKind::Int);
                assert_eq!(result_type.get_display_name(), "int");
            }
            _ => unreachable!(),
        }

        #[cfg(all(feature="clang_6_0", target_os="macos"))]
        fn test_get_mangled_objc_names(entity: &Entity) {
            let names = vec!["_OBJC_CLASS_$_A".into(), "_OBJC_METACLASS_$_A".into()];
            assert_eq!(entity.get_mangled_objc_names(), Some(names));
        }

        #[cfg(all(feature="clang_6_0", not(target_os="macos")))]
        fn test_get_mangled_objc_names(entity: &Entity) {
            let names = vec!["_OBJC_CLASS_A".into(), "_OBJC_METACLASS_A".into()];
            assert_eq!(entity.get_mangled_objc_names(), Some(names));
        }

        #[cfg(not(feature="clang_6_0"))]
        fn test_get_mangled_objc_names(_: &Entity) { }

        test_get_mangled_objc_names(&entities[1]);
    });

    let source = "
        struct x {
            char y;
            char z;
        }
    ";

    with_entity(&clang, source, |e| {
        let children = e.get_children();

        #[cfg(feature="clang_3_7")]
        fn test_get_offset_of_field(fields: &[Entity]) {
            assert_eq!(fields[0].get_offset_of_field(), Ok(0));
            assert_eq!(fields[1].get_offset_of_field(), Ok(8));
        }

        #[cfg(not(feature="clang_3_7"))]
        fn test_get_offset_of_field(_: &[Entity]) {}

        test_get_offset_of_field(&children[0].get_children());
    });

    let source = "
        const int x = 0;
    ";

    with_entity(&clang, source, |e| {
        let children = e.get_children();

        #[cfg(feature="clang_7_0")]
        fn test_is_invalid_declaration(entity: Entity) {
            assert_eq!(entity.is_invalid_declaration(), false);
        }

        #[cfg(not(feature="clang_7_0"))]
        fn test_is_invalid_declaration(_: Entity) {}

        test_is_invalid_declaration(children[0]);
    });

    let source = "
        int main() {
            return 0;
        }
    ";

    with_entity(&clang, source, |e| {
        let children = e.get_children();

        #[cfg(feature="clang_7_0")]
        fn test_pretty_printer(entity: Entity) {
            let s = entity.get_pretty_printer()
                .set_indentation_amount(1)
                .set_flag(PrintingPolicyFlag::IncludeNewlines, true)
                .set_flag(PrintingPolicyFlag::IncludeTagDefinition, true)
                .set_flag(PrintingPolicyFlag::PolishForDeclaration, false)
                .set_flag(PrintingPolicyFlag::PrintAnonymousTagLocations, false)
                .set_flag(PrintingPolicyFlag::PrintConstantArraySizeAsWritten, true)
                .set_flag(PrintingPolicyFlag::PrintConstantsAsWritten, true)
                .set_flag(PrintingPolicyFlag::PrintFullyQualifiedName, true)
                .set_flag(PrintingPolicyFlag::SuppressImplicitBase, true)
                .set_flag(PrintingPolicyFlag::SuppressInitializers, false)
                .set_flag(PrintingPolicyFlag::SuppressLifetimeQualifiers, false)
                .set_flag(PrintingPolicyFlag::SuppressScope, false)
                .set_flag(PrintingPolicyFlag::SuppressSpecifiers, false)
                .set_flag(PrintingPolicyFlag::SuppressStrongLifetime, false)
                .set_flag(PrintingPolicyFlag::SuppressTagKeyword, true)
                .set_flag(PrintingPolicyFlag::SuppressTemplateArgsInCXXConstructors, false)
                .set_flag(PrintingPolicyFlag::SuppressUnwrittenScope, false)
                .set_flag(PrintingPolicyFlag::UseAlignof, true)
                .set_flag(PrintingPolicyFlag::UseBool, true)
                .set_flag(PrintingPolicyFlag::UseHalf, false)
                .set_flag(PrintingPolicyFlag::UseMsWchar, false)
                .set_flag(PrintingPolicyFlag::UseMsvcFormatting, false)
                .set_flag(PrintingPolicyFlag::UseRestrict, true)
                .set_flag(PrintingPolicyFlag::UseTerseOutput, false)
                .set_flag(PrintingPolicyFlag::UseUnderscoreAlignof, false)
                .set_flag(PrintingPolicyFlag::UseVoidForZeroParams, true)
                .print();
            assert_eq!(s, "int main() {\n  return 0;\n}\n");
        }

        #[cfg(not(feature="clang_7_0"))]
        fn test_pretty_printer(_: Entity) {}

        test_pretty_printer(children[0]);
    });

    let source = "
        @interface Foo
        - @property NSString *x;
        @end
    ";

    with_translation_unit(&clang, "test.mm", source, &[], |_, _, tu| {
        let children = tu.get_entity().get_children();

        #[cfg(feature="clang_8_0")]
        fn test_get_objc_getter_setter_name(properties: &[Entity]) {
            assert_eq!(properties[0].get_objc_getter_name().as_ref().map(|s| s.as_ref()), Some("x"));
            assert_eq!(properties[0].get_objc_setter_name().as_ref().map(|s| s.as_ref()), Some("setX:"));
        }

        #[cfg(not(feature="clang_8_0"))]
        fn test_get_objc_getter_setter_name(_: &[Entity]) {}

        test_get_objc_getter_setter_name(&children[1].get_children());
    });

    // Index _____________________________________

    let mut index = Index::new(&clang, false, false);

    let mut options = ThreadOptions::default();
    assert_eq!(index.get_thread_options(), options);

    options.editing = true;
    index.set_thread_options(options);
    assert_eq!(index.get_thread_options(), options);

    // TranslationUnit ___________________________

    with_translation_unit(&clang, "test.cpp", "int a = 322;", &[], |d, _, tu| {
        let file = d.join("test.cpp.gch");
        tu.save(&file).unwrap();
        let index = Index::new(&clang, false, false);
        let _ = TranslationUnit::from_ast(&index, &file).unwrap();
    });

    with_temporary_file("test.cpp", "int a = 322;", |_, f| {
        let index = Index::new(&clang, false, false);
        let _ = index.parser(f).unsaved(&[Unsaved::new(f, "int a = 644;")]).parse().unwrap();
    });

    with_translation_unit(&clang, "test.cpp", "int a = 322;", &[], |d, _, tu| {
        assert_eq!(tu.get_file(d.join("test.c")), None);
    });

    with_translation_unit(&clang, "test.cpp", "int a = 322;", &[], |_, _, tu| {
        let usage = tu.get_memory_usage();
        assert_eq!(usage.get(&MemoryUsage::Selectors), Some(&0));
    });

    with_translation_unit(&clang, "test.cpp", "int a = 322;", &[], |_, f, tu| {
        let _ = tu.reparse(&[Unsaved::new(f, "int a = 644;")]).unwrap();
    });

    // Type ______________________________________

    with_entity(&clang, "int a = 322;", |e| {
        assert_eq!(e.get_type(), None);

        let type_ = e.get_children()[0].get_type().unwrap();
        assert_eq!(type_.get_display_name(), "int");
        assert_eq!(type_.get_kind(), TypeKind::Int);
    });

    let source = "
        int integer = 322;
        int function(int argument) { return argument; }
    ";

    with_types(&clang, source, |ts| {
        assert_eq!(ts[0].get_argument_types(), None);
        assert_eq!(ts[0].get_result_type(), None);

        assert_eq!(ts[1].get_argument_types(), Some(vec![ts[0]]));
        assert_eq!(ts[1].get_result_type(), Some(ts[0]));
    });

    let source = "
        template <typename T> struct A { T a; int b; };
        typedef A<int> B;
        struct C { int a; int b; };
    ";

    with_types(&clang, source, |ts| {
        assert_eq!(ts[0].get_alignof(), Err(AlignofError::Incomplete));
        assert_eq!(ts[0].get_offsetof("b"), Err(OffsetofError::Parent));
        assert_eq!(ts[0].get_sizeof(), Err(SizeofError::Incomplete));

        let size = mem::size_of::<c_int>();
        assert_eq!(ts[1].get_alignof(), Ok(size));
        assert_eq!(ts[1].get_offsetof("b"), Ok(size * 8));
        assert_eq!(ts[1].get_sizeof(), Ok(size * 2));
    });

    let source = "
        int integer = 322;
        void a() { }
    ";

    with_types(&clang, source, |ts| {
        assert_eq!(ts[0].get_calling_convention(), None);
        assert_eq!(ts[1].get_calling_convention(), Some(CallingConvention::Cdecl));
    });

    let source = "
        int integer;
        typedef int Integer;
    ";

    with_types(&clang, source, |ts| {
        assert_eq!(ts[0].get_canonical_type(), ts[0]);
        assert_eq!(ts[1].get_canonical_type(), ts[0]);
    });

    let source = "
        struct Struct { int member; };
        int Struct::*pointer = &Struct::member;
    ";

    with_types(&clang, source, |ts| {
        assert_eq!(ts[0].get_class_type(), None);
        assert_eq!(ts[1].get_class_type(), Some(ts[0]));
    });

    let source = "
        typedef int Integer;
        Integer integer;
    ";

    with_entity(&clang, source, |e| {
        let types = e.get_children().iter().map(|e| e.get_type().unwrap()).collect::<Vec<_>>();
        assert_eq!(types[0].get_declaration(), Some(e.get_children()[0]));
        assert_eq!(types[1].get_declaration(), Some(e.get_children()[0]));
    });

    let source = "
        class A { };
        int A;
        class A a;
    ";

    with_types(&clang, source, |ts| {
        #[cfg(feature="clang_3_9")]
        fn test_get_elaborated_type<'tu>(types: &[Type<'tu>]) {
            assert_eq!(types[2].get_elaborated_type(), Some(types[0]));
        }

        #[cfg(not(feature="clang_3_9"))]
        fn test_get_elaborated_type<'tu>(_: &[Type<'tu>]) { }

        test_get_elaborated_type(&ts);
    });

    let source = "
        int integer = 322;
        int array[3] = { 3, 2, 2 };
    ";

    with_types(&clang, source, |ts| {
        assert_eq!(ts[0].get_element_type(), None);
        assert_eq!(ts[0].get_size(), None);

        assert_eq!(ts[1].get_element_type(), Some(ts[0]));
        assert_eq!(ts[1].get_size(), Some(3));
    });

    let source = "
        void a();
        void b() throw();
        void c() throw(int);
    ";

    with_types(&clang, source, |ts| {
        #[cfg(feature="clang_5_0")]
        fn test_get_exception_specification(ts: &[Type]) {
            assert_eq!(ts[0].get_exception_specification(), None);
            assert_eq!(ts[1].get_exception_specification(), Some(ExceptionSpecification::DynamicNone));
            assert_eq!(ts[2].get_exception_specification(), Some(ExceptionSpecification::Dynamic));
        }

        #[cfg(not(feature="clang_5_0"))]
        fn test_get_exception_specification(_: &[Type]) { }

        test_get_exception_specification(&ts[..]);
    });

    let source = "
        struct A { int a, b, c; };
    ";

    with_entity(&clang, source, |e| {
        #[cfg(feature="clang_3_7")]
        fn test_get_fields<'tu>(entity: Entity<'tu>) {
            assert_eq!(entity.get_type().unwrap().get_fields(), Some(entity.get_children()));
        }

        #[cfg(not(feature="clang_3_7"))]
        fn test_get_fields<'tu>(_: Entity<'tu>) { }

        test_get_fields(e.get_children()[0]);
    });

    let source = "
        int integer = 322;
        int* pointer = &integer;
    ";

    with_types(&clang, source, |ts| {
        assert_eq!(ts[0].get_pointee_type(), None);
        assert_eq!(ts[1].get_pointee_type(), Some(ts[0]));
    });

    let source = "
        class Class {
            void a();
            void b() &;
            void c() &&;
        };
    ";

    with_types(&clang, source, |ts| {
        let types = ts[0].get_declaration().unwrap().get_children().into_iter().map(|c| {
            c.get_type().unwrap()
        }).collect::<Vec<_>>();

        assert_eq!(types[0].get_ref_qualifier(), None);
        assert_eq!(types[1].get_ref_qualifier(), Some(RefQualifier::LValue));
        assert_eq!(types[2].get_ref_qualifier(), Some(RefQualifier::RValue));
    });

    let source = "
        template <typename T, int I> class Class { int member; };
        int integer = 322;
        template <> class Class<int, 322> { int member; };
    ";

    with_types(&clang, source, |ts| {
        assert_eq!(ts[0].get_template_argument_types(), None);
        assert_eq!(ts[1].get_template_argument_types(), Some(vec![Some(ts[0]), None]));
    });

    let source = "
        int a;
        typedef int Integer;
        Integer b;
    ";

    with_types(&clang, source, |ts| {
        #[cfg(feature="clang_5_0")]
        fn test_get_typedef_name(ts: &[Type]) {
            assert_eq!(ts[0].get_typedef_name(), None);
            assert_eq!(ts[1].get_typedef_name(), Some("Integer".into()));
            assert_eq!(ts[2].get_typedef_name(), Some("Integer".into()));
        }

        #[cfg(not(feature="clang_5_0"))]
        fn test_get_typedef_name(_: &[Type]) { }

        test_get_typedef_name(&ts[..]);
    });

    let source = "
        int a = 322;
        const int b = 322;
        int* __restrict__ c = &a;
        volatile int d = 322;
    ";

    with_types(&clang, source, |ts| {
        macro_rules! qualifiers {
            ($type_:expr, $c:expr, $r:expr, $v:expr) => ({
                assert_eq!($type_.is_const_qualified(), $c);
                assert_eq!($type_.is_restrict_qualified(), $r);
                assert_eq!($type_.is_volatile_qualified(), $v);
            });
        }

        qualifiers!(ts[0], false, false, false);
        qualifiers!(ts[1], true, false, false);
        qualifiers!(ts[2], false, true, false);
        qualifiers!(ts[3], false, false, true);
    });

    let source = "
        struct A { };
        struct B { ~B() { } };
    ";

    with_types(&clang, source, |ts| {
        assert!(ts[0].is_pod());
        assert!(!ts[1].is_pod());
    });

    let source = "
        void a() { }
        void b(...) { }
    ";

    with_types(&clang, source, |ts| {
        assert!(!ts[0].is_variadic());
        assert!(ts[1].is_variadic());
    });

    let source = "
        @class C<T>;
        @protocol P
        @end
        C<C*><P> *x;
        C* y;
    ";

    with_translation_unit(&clang, "test.mm", source, &[], |_, _, tu| {
        let children = tu.get_entity().get_children();

        #[cfg(feature="clang_8_0")]
        fn test_objc_object_type(e: &[Entity]) {
            let ty = e[3].get_type().unwrap().get_pointee_type().unwrap();
            assert_eq!(ty.get_objc_object_base_type(), Some(e[1].get_type().unwrap()));
            let protocols = ty.get_objc_protocol_declarations();
            assert_eq!(protocols.len(), 1);
            assert_eq!(protocols[0], e[2]);
            let args = ty.get_objc_type_arguments();
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], e[4].get_type().unwrap());
        }

        #[cfg(not(feature="clang_8_0"))]
        fn test_objc_object_type(_: &[Entity]) {}

        test_objc_object_type(&children);
    });

    let source = "
        void f(void)  __attribute__((availability(ios,unavailable))) __attribute__((availability(macos,introduced=10.1.1,deprecated=10.2,obsoleted=11)));
    ";
    with_entity(&clang, source, |e| {
        let platform_availability = e.get_children().first().unwrap().get_platform_availability().unwrap();
        assert_eq!(platform_availability, vec![
            PlatformAvailability {
                platform: "ios".to_string(),
                unavailable: true,
                introduced: None,
                deprecated: None,
                obsoleted: None,
                message: None,
            },
            PlatformAvailability {
                platform: "macos".to_string(),
                unavailable: false,
                introduced: Some(Version { x: 10, y: Some(1), z: Some(1) }),
                deprecated: Some(Version { x: 10, y: Some(2), z: None }),
                obsoleted: Some(Version { x: 11, y: None, z: None }),
                message: None,
            },
        ])
    });

    // Usr _______________________________________

    let class = Usr::from_objc_class("A");
    assert_eq!(class, Usr("c:objc(cs)A".into()));

    assert_eq!(Usr::from_objc_category("A", "B"), Usr("c:objc(cy)A@B".into()));
    assert_eq!(Usr::from_objc_ivar(&class, "B"), Usr("c:objc(cs)A@B".into()));
    assert_eq!(Usr::from_objc_method(&class, "B", true), Usr("c:objc(cs)A(im)B".into()));
    assert_eq!(Usr::from_objc_method(&class, "B", false), Usr("c:objc(cs)A(cm)B".into()));
    assert_eq!(Usr::from_objc_property(&class, "B"), Usr("c:objc(cs)A(py)B".into()));
    assert_eq!(Usr::from_objc_protocol("A"), Usr("c:objc(pl)A".into()));
}
