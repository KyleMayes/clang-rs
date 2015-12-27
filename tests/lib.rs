extern crate clang;
extern crate libc;
extern crate uuid;

use std::env;
use std::fs;
use std::mem;
use std::io::{Write};
use std::path::{Path, PathBuf};

use clang::*;

use libc::{c_double, c_int};

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
        let index = Index::new(clang, false, false);
        let options = ParseOptions::default();
        let tu = TranslationUnit::from_source(&index, file, arguments, &[], options).unwrap();
        f(d, &file, tu);
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

#[test]
fn test() {
    let clang = Clang::new().unwrap();

    // CompilationDatabase _______________________

    let source = r#"[
        {
          "directory": "/tmp",
          "command": "cc div0.c",
          "file": "/tmp/div0.c"
        },
        {
          "directory": "/tmp",
          "command": "cc -DFOO div1.c",
          "file": "/tmp/div1.c"
        }
    ]"#;

    // FIXME: possible libclang bug on Windows
    with_temporary_file("compile_commands.json", source, |d, _| {
        #[cfg(not(target_os="windows"))]
        fn test_compilation_database(cd: CompilationDatabase) {
            assert_eq!(cd.get_all_commands().len(), 2);

            let commands = cd.get_commands("/tmp/div0.c");
            assert_eq!(commands.len(), 1);

            assert_eq!(commands[0].get_arguments(), &["cc", "div0.c"]);

            let commands = cd.get_commands("/tmp/div1.c");
            assert_eq!(commands.len(), 1);

            assert_eq!(commands[0].get_arguments(), &["cc", "-DFOO", "div1.c"]);
        }

        #[cfg(target_os="windows")]
        fn test_compilation_database(_: CompilationDatabase) { }

        test_compilation_database(CompilationDatabase::from_directory(&clang, d).unwrap());
    });

    // CompletionString __________________________

    let source = "
        struct A {
            /// \\brief An integer field.
            int a;
            int b;
            int c;
        };
        void b() { A a; a. }
    ";

    with_temporary_file("test.cpp", source, |_, f| {
        let index = Index::new(&clang, false, false);
        let mut options = ParseOptions::default();
        options.briefs_in_completion_results = true;
        let tu = TranslationUnit::from_source(&index, f, &[], &[], options).unwrap();

        let mut options = CompletionOptions::default();
        options.briefs = true;
        let results = tu.complete(f, 8, 27, &[], options);
        assert_eq!(results.get_container_kind(), Some((EntityKind::StructDecl, false)));
        assert!(results.get_diagnostics(&tu).is_empty());
        assert_eq!(results.get_usr(), Some(Usr("c:@S@A".into())));

        let context = results.get_context().unwrap();
        assert!(!context.all_types);
        assert!(!context.all_values);
        assert!(!context.class_type_values);
        assert!(context.dot_members);
        assert!(!context.arrow_members);
        assert!(!context.enum_tags);
        assert!(!context.union_tags);
        assert!(!context.struct_tags);
        assert!(!context.class_names);
        assert!(!context.namespaces);
        assert!(!context.nested_name_specifiers);
        assert!(!context.macro_names);
        assert!(!context.natural_language);
        assert!(!context.objc_object_values);
        assert!(!context.objc_selector_values);
        assert!(!context.objc_property_members);
        assert!(!context.objc_interfaces);
        assert!(!context.objc_protocols);
        assert!(!context.objc_categories);
        assert!(!context.objc_instance_messages);
        assert!(!context.objc_class_messages);
        assert!(!context.objc_selector_names);

        let mut results = results.get_results();
        assert_eq!(results.len(), 6);
        results.sort();

        assert_eq!(results[0].get_kind(), EntityKind::Method);
        let string = results[0].get_string();
        assert_eq!(string.get_annotations(), &[] as &[&str]);
        assert_eq!(string.get_availability(), Availability::Available);
        assert_eq!(string.get_comment_brief(), None);
        assert_eq!(string.get_chunks(), &[
            CompletionChunk::ResultType("A &".into()),
            CompletionChunk::TypedText("operator=".into()),
            CompletionChunk::LeftParenthesis("(".into()),
            CompletionChunk::Placeholder("const A &".into()),
            CompletionChunk::RightParenthesis(")".into()),
        ]);
        assert_eq!(string.get_parent_name(), Some("A".into()));
        assert_eq!(string.get_priority(), 34);
        assert_eq!(string.get_typed_text(), Some("operator=".into()));

        assert_eq!(results[1].get_kind(), EntityKind::Destructor);
        let string = results[1].get_string();
        assert_eq!(string.get_annotations(), &[] as &[&str]);
        assert_eq!(string.get_availability(), Availability::Available);
        assert_eq!(string.get_chunks(), &[
            CompletionChunk::ResultType("void".into()),
            CompletionChunk::TypedText("~A".into()),
            CompletionChunk::LeftParenthesis("(".into()),
            CompletionChunk::RightParenthesis(")".into()),
        ]);
        assert_eq!(string.get_parent_name(), Some("A".into()));
        assert_eq!(string.get_priority(), 34);
        assert_eq!(string.get_typed_text(), Some("~A".into()));

        assert_eq!(results[2].get_kind(), EntityKind::FieldDecl);
        let string = results[2].get_string();
        assert_eq!(string.get_annotations(), &[] as &[&str]);
        assert_eq!(string.get_availability(), Availability::Available);
        assert_eq!(string.get_comment_brief(), Some("An integer field.".into()));
        assert_eq!(string.get_chunks(), &[
            CompletionChunk::ResultType("int".into()),
            CompletionChunk::TypedText("a".into()),
        ]);
        assert_eq!(string.get_parent_name(), Some("A".into()));
        assert_eq!(string.get_priority(), 35);
        assert_eq!(string.get_typed_text(), Some("a".into()));

        assert_eq!(results[3].get_kind(), EntityKind::FieldDecl);
        let string = results[3].get_string();
        assert_eq!(string.get_annotations(), &[] as &[&str]);
        assert_eq!(string.get_availability(), Availability::Available);
        assert_eq!(string.get_comment_brief(), None);
        assert_eq!(string.get_chunks(), &[
            CompletionChunk::ResultType("int".into()),
            CompletionChunk::TypedText("b".into()),
        ]);
        assert_eq!(string.get_parent_name(), Some("A".into()));
        assert_eq!(string.get_priority(), 35);
        assert_eq!(string.get_typed_text(), Some("b".into()));

        assert_eq!(results[4].get_kind(), EntityKind::FieldDecl);
        let string = results[4].get_string();
        assert_eq!(string.get_annotations(), &[] as &[&str]);
        assert_eq!(string.get_availability(), Availability::Available);
        assert_eq!(string.get_comment_brief(), None);
        assert_eq!(string.get_chunks(), &[
            CompletionChunk::ResultType("int".into()),
            CompletionChunk::TypedText("c".into()),
        ]);
        assert_eq!(string.get_parent_name(), Some("A".into()));
        assert_eq!(string.get_priority(), 35);
        assert_eq!(string.get_typed_text(), Some("c".into()));

        assert_eq!(results[5].get_kind(), EntityKind::StructDecl);
        let string = results[5].get_string();
        assert_eq!(string.get_annotations(), &[] as &[&str]);
        assert_eq!(string.get_availability(), Availability::Available);
        assert_eq!(string.get_comment_brief(), None);
        assert_eq!(string.get_chunks(), &[
            CompletionChunk::TypedText("A".into()),
            CompletionChunk::Text("::".into()),
        ]);
        assert_eq!(string.get_parent_name(), Some("A".into()));
        assert_eq!(string.get_priority(), 75);
        assert_eq!(string.get_typed_text(), Some("A".into()));
    });

    // File ______________________________________

    with_file(&clang, "int a = 322;", |p, f| {
        with_file(&clang, "int a = 322;", |_, g| assert!(f.get_id() != g.get_id()));
        assert_eq!(f.get_path(), p.to_path_buf());
        assert_eq!(f.get_skipped_ranges(), &[]);
        assert!(f.get_time() != 0);
        assert!(!f.is_include_guarded());
    });

    let source = "
        #if 0
        int skipped = 32;
        #endif
        int unskipped = 32;
    ";

    with_temporary_file("test.cpp", source, |_, f| {
        let index = Index::new(&clang, false, false);
        let mut options = ParseOptions::default();
        options.detailed_preprocessing_record = true;
        let tu = TranslationUnit::from_source(&index, f, &[], &[], options).unwrap();

        let file = tu.get_file(f).unwrap();
        assert_eq!(file.get_skipped_ranges(), &[range!(file, 2, 10, 4, 15)]);
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
        options.source_location = false;
        options.option = false;

        let file = tu.get_file(f).unwrap();

        let diagnostics = tu.get_diagnostics();
        assert_eq!(diagnostics.len(), 3);

        let text = "implicit conversion turns floating-point number into integer: 'float' to 'int'";
        assert_eq!(diagnostics[0].format(options), format!("warning: {}", text));
        assert!(diagnostics[0].get_children().is_empty());
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
        assert!(diagnostics[1].get_children().is_empty());
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
        assert!(diagnostics[2].get_children().is_empty());
        assert_eq!(diagnostics[2].get_fix_its(), &[FixIt::Replacement(range, ".i = ".into())]);
        assert_eq!(diagnostics[2].get_location(), range.get_start());
        assert_eq!(diagnostics[2].get_ranges(), &[]);
        assert_eq!(diagnostics[2].get_severity(), Severity::Warning);
        assert_eq!(diagnostics[2].get_text(), text);
    });

    // Entity ____________________________________

    with_translation_unit(&clang, "test.cpp", "int a = 322;", &[], |_, f, tu| {
        #[cfg(any(feature="clang_3_6", feature="clang_3_7"))]
        fn test_get_mangled_name<'tu>(entity: Entity<'tu>) {
            assert_eq!(entity.get_mangled_name(), None);

            let children = entity.get_children();
            assert_eq!(children[0].get_mangled_name(), Some("_Z1a".into()));
        }

        #[cfg(not(any(feature="clang_3_6", feature="clang_3_7")))]
        fn test_get_mangled_name<'tu>(_: Entity<'tu>) { }

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

        test_get_mangled_name(entity);
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

        assert_eq!(children[1].get_bit_field_width(), Some(322));
        assert_eq!(children[1].get_name(), Some("i".into()));
        assert_eq!(children[1].get_display_name(), Some("i".into()));
        assert!(children[1].is_bit_field());
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

        assert_eq!(children[1].get_enum_constant_value(), None);
        assert_eq!(children[1].get_enum_underlying_type(), Some(children[0].get_type().unwrap()));

        let children = children[1].get_children();
        assert_eq!(children.len(), 2);

        assert_eq!(children[0].get_enum_constant_value(), Some((322, 322)));
        assert_eq!(children[1].get_enum_constant_value(), Some((644, 644)));
    });

    let files = &[
        ("test.hpp", ""),
        ("test.cpp", "#include \"test.hpp\""),
    ];

    with_temporary_files(files, |_, fs| {
        let index = Index::new(&clang, false, false);
        let mut options = ParseOptions::default();
        options.detailed_preprocessing_record = true;
        let tu = TranslationUnit::from_source(&index, &fs[1], &[], &[], options).unwrap();

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
        #[cfg(any(feature="clang_3_6", feature="clang_3_7"))]
        fn test_get_storage_class<'tu>(entity: Entity<'tu>) {
            assert_eq!(entity.get_storage_class(), None);

            let children = entity.get_children();
            assert_eq!(children[0].get_storage_class(), Some(StorageClass::None));
            assert_eq!(children[1].get_storage_class(), Some(StorageClass::Static));
        }

        #[cfg(not(any(feature="clang_3_6", feature="clang_3_7")))]
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
        #[cfg(any(feature="clang_3_6", feature="clang_3_7"))]
        fn test_get_template_arguments<'tu>(children: &[Entity<'tu>]) {
            assert_eq!(children[0].get_template_arguments(), None);
            assert_eq!(children[1].get_template_arguments(), None);
            assert_eq!(children[2].get_template_arguments(), Some(vec![
                TemplateArgument::Type(children[0].get_type().unwrap()),
                TemplateArgument::Integral(322, 322),
            ]));
        }

        #[cfg(not(any(feature="clang_3_6", feature="clang_3_7")))]
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
        let index = Index::new(&clang, false, false);
        let arguments = &["-fmodules"];
        let options = ParseOptions::default();
        let tu = TranslationUnit::from_source(&index, &fs[2], arguments, &[], options).unwrap();

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

    with_file(&clang, "int a = 322;", |_, f| {
        let range = range!(f, 1, 5, 1, 6);
        assert_location_eq!(range.get_end().get_spelling_location(), f, 1, 6, 5);
        assert_location_eq!(range.get_start().get_spelling_location(), f, 1, 5, 4);
    });

    // Tokens ____________________________________

    // FIXME: possible libclang bug on Windows
    with_translation_unit(&clang, "test.cpp", "int a = 322; ", &[], |_, f, tu| {
        #[cfg(not(target_os="windows"))]
        fn test_annotate<'tu>(tu: &'tu TranslationUnit<'tu>, tokens: &[Token<'tu>]) {
            let entity = tu.get_entity().get_children()[0];

            assert_eq!(tu.annotate(tokens), &[
                Some(entity), Some(entity), None, None, Some(entity.get_children()[0])
            ]);
        }

        #[cfg(target_os="windows")]
        fn test_annotate<'tu>(_: &'tu TranslationUnit<'tu>, _: &[Token<'tu>]) { }

        let file = tu.get_file(f).unwrap();

        let tokens = range!(file, 1, 1, 1, 13).tokenize();
        assert_eq!(tokens.len(), 5);

        assert_eq!(tokens[0].get_kind(), TokenKind::Keyword);
        assert_location_eq!(tokens[0].get_location().get_spelling_location(), file, 1, 1, 0);
        assert_eq!(tokens[0].get_range(), range!(file, 1, 1, 1, 4));
        assert_eq!(tokens[0].get_spelling(), "int");

        assert_eq!(tokens[1].get_kind(), TokenKind::Identifier);
        assert_location_eq!(tokens[1].get_location().get_spelling_location(), file, 1, 5, 4);
        assert_eq!(tokens[1].get_range(), range!(file, 1, 5, 1, 6));
        assert_eq!(tokens[1].get_spelling(), "a");

        assert_eq!(tokens[2].get_kind(), TokenKind::Punctuation);
        assert_location_eq!(tokens[2].get_location().get_spelling_location(), file, 1, 7, 6);
        assert_eq!(tokens[2].get_range(), range!(file, 1, 7, 1, 8));
        assert_eq!(tokens[2].get_spelling(), "=");

        assert_eq!(tokens[3].get_kind(), TokenKind::Literal);
        assert_location_eq!(tokens[3].get_location().get_spelling_location(), file, 1, 9, 8);
        assert_eq!(tokens[3].get_range(), range!(file, 1, 9, 1, 12));
        assert_eq!(tokens[3].get_spelling(), "322");

        assert_eq!(tokens[4].get_kind(), TokenKind::Punctuation);
        assert_location_eq!(tokens[4].get_location().get_spelling_location(), file, 1, 12, 11);
        assert_eq!(tokens[4].get_range(), range!(file, 1, 12, 1, 13));
        assert_eq!(tokens[4].get_spelling(), ";");

        test_annotate(&tu, &tokens);
    });

    // TranslationUnit ___________________________

    //- from_ast ---------------------------------
    //- save -------------------------------------

    with_translation_unit(&clang, "test.cpp", "int a = 322;", &[], |d, _, tu| {
        let file = d.join("test.cpp.gch");
        tu.save(&file).unwrap();
        let index = Index::new(&clang, false, false);
        let _ = TranslationUnit::from_ast(&index, &file).unwrap();
    });

    //- from_source ------------------------------

    with_temporary_file("test.cpp", "int a = 322;", |_, f| {
        let index = Index::new(&clang, false, false);
        let unsaved = &[Unsaved::new(f, "int a = 644;")];
        let options = ParseOptions::default();
        let _ = TranslationUnit::from_source(&index, f, &[], unsaved, options).unwrap();
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
        __attribute__((vectorcall)) void b() { }
    ";

    with_types(&clang, source, |ts| {
        assert_eq!(ts[0].get_calling_convention(), None);
        assert_eq!(ts[1].get_calling_convention(), Some(CallingConvention::Cdecl));
        assert_eq!(ts[2].get_calling_convention(), Some(CallingConvention::Vectorcall));
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

    // Usr _______________________________________

    let class = Usr::from_objc_class("A");
    assert_eq!(class, Usr("c:objc(cs)A".into()));

    assert_eq!(Usr::from_objc_category("A", "B"), Usr("c:objc(cy)A@B".into()));
    assert_eq!(Usr::from_objc_ivar(&class, "B"), Usr("c:objc(cs)A@B".into()));
    assert_eq!(Usr::from_objc_method(&class, "B", true), Usr("c:objc(cs)A(im)B".into()));
    assert_eq!(Usr::from_objc_method(&class, "B", false), Usr("c:objc(cs)A(cm)B".into()));
    assert_eq!(Usr::from_objc_property(&class, "B"), Usr("c:objc(cs)A(py)B".into()));
    assert_eq!(Usr::from_objc_protocol("A"), Usr("c:objc(pl)A".into()));

    // sonar _____________________________________

    let source = "
        enum A { AA, AB, AC };
        typedef enum { BA = -1, BB = -2, BC = -3 } B;
        typedef enum C { CA = 1, CB = 2, CC = 3 } C;
        struct D { int d; };
        typedef struct { int e; } E;
        typedef struct F { int f; } F;
        union G { int a; float b; double c; };
        typedef union { int a; float b; double c; } H;
        typedef union I { int a; float b; double c; } I;
    ";

    with_translation_unit(&clang, "test.c", source, &[], |_, _, tu| {
        let enums = sonar::find_enums(&tu);
        assert_eq!(enums.len(), 3);

        assert_eq!(enums[0].get_name(), "A");
        assert!(!enums[0].is_signed());
        assert_eq!(enums[0].get_unsigned_constants(), &[
            ("AA".into(), 0), ("AB".into(), 1), ("AC".into(), 2)
        ]);

        assert_eq!(enums[1].get_name(), "B");
        assert!(enums[1].is_signed());
        assert_eq!(enums[1].get_signed_constants(), &[
            ("BA".into(), -1), ("BB".into(), -2), ("BC".into(), -3)
        ]);

        assert_eq!(enums[2].get_name(), "C");
        assert!(!enums[2].is_signed());
        assert_eq!(enums[2].get_unsigned_constants(), &[
            ("CA".into(), 1), ("CB".into(), 2), ("CC".into(), 3)
        ]);

        let structs = sonar::find_structs(&tu);
        assert_eq!(structs.len(), 3);

        assert_eq!(structs[0].get_name(), "D");

        assert_eq!(structs[0].get_fields().len(), 1);
        assert_eq!(structs[0].get_fields()[0].get_name(), Some("d".into()));

        assert_eq!(structs[1].get_name(), "E");

        assert_eq!(structs[1].get_fields().len(), 1);
        assert_eq!(structs[1].get_fields()[0].get_name(), Some("e".into()));

        assert_eq!(structs[2].get_name(), "F");

        assert_eq!(structs[2].get_fields().len(), 1);
        assert_eq!(structs[2].get_fields()[0].get_name(), Some("f".into()));

        let unions = sonar::find_unions(&tu);
        assert_eq!(unions.len(), 3);

        assert_eq!(unions[0].get_name(), "G");
        assert_eq!(unions[0].get_size(), mem::size_of::<c_double>());

        assert_eq!(unions[0].get_fields().len(), 3);
        assert_eq!(unions[0].get_fields()[0].get_name(), Some("a".into()));
        assert_eq!(unions[0].get_fields()[1].get_name(), Some("b".into()));
        assert_eq!(unions[0].get_fields()[2].get_name(), Some("c".into()));

        assert_eq!(unions[1].get_name(), "H");
        assert_eq!(unions[1].get_size(), mem::size_of::<c_double>());

        assert_eq!(unions[1].get_fields().len(), 3);
        assert_eq!(unions[1].get_fields()[0].get_name(), Some("a".into()));
        assert_eq!(unions[1].get_fields()[1].get_name(), Some("b".into()));
        assert_eq!(unions[1].get_fields()[2].get_name(), Some("c".into()));

        assert_eq!(unions[2].get_name(), "I");
        assert_eq!(unions[2].get_size(), mem::size_of::<c_double>());

        assert_eq!(unions[2].get_fields().len(), 3);
        assert_eq!(unions[2].get_fields()[0].get_name(), Some("a".into()));
        assert_eq!(unions[2].get_fields()[1].get_name(), Some("b".into()));
        assert_eq!(unions[2].get_fields()[2].get_name(), Some("c".into()));
    });
}
