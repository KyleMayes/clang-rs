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

//================================================
// Functions
//================================================

fn with_file<'c, F: FnOnce(&Path, File)>(clang: &'c Clang, contents: &str, f: F) {
    with_translation_unit(clang, "test.h", contents, &[], |_, file, tu| {
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
            SourceRange::new(file.get_location(2, 44), file.get_location(2, 49)),
            SourceRange::new(file.get_location(2, 37), file.get_location(2, 43)),
        ]);
        assert_eq!(diagnostics[0].get_severity(), Severity::Warning);
        assert_eq!(diagnostics[0].get_text(), text);

        let text = "missing 'typename' prior to dependent type name 'T::U'";
        assert_eq!(diagnostics[1].format(options), format!("error: {}", text));
        assert_eq!(diagnostics[1].get_fix_its(), &[
            FixIt::Insertion(file.get_location(3, 50), "typename ".into())
        ]);
        assert_eq!(diagnostics[1].get_location(), file.get_location(3, 50));
        assert_eq!(diagnostics[1].get_ranges(), &[
            SourceRange::new(file.get_location(3, 50), file.get_location(3, 54))
        ]);
        assert_eq!(diagnostics[1].get_severity(), Severity::Error);
        assert_eq!(diagnostics[1].get_text(), text);

        let text = "use of GNU old-style field designator extension";
        assert_eq!(diagnostics[2].format(options), format!("warning: {}", text));
        let range = SourceRange::new(file.get_location(4, 50), file.get_location(4, 52));
        assert_eq!(diagnostics[2].get_fix_its(), &[FixIt::Replacement(range, ".i = ".into())]);
        assert_eq!(diagnostics[2].get_location(), range.get_start());
        assert_eq!(diagnostics[2].get_ranges(), &[]);
        assert_eq!(diagnostics[2].get_severity(), Severity::Warning);
        assert_eq!(diagnostics[2].get_text(), text);
    });

    // Index _____________________________________

    let mut index = Index::new(&clang, false, false);

    let mut priority = BackgroundPriority { editing: false, indexing: false };
    assert_eq!(index.get_background_priority(), priority);

    priority.editing = true;
    index.set_background_priority(priority);
    assert_eq!(index.get_background_priority(), priority);

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
        #line 322 \"presumed.h\"
        int add(int left, int right) { return ADD(left, right); }
    ";

    with_file(&clang, source, |_, f| {
        let location = f.get_location(3, 51);
        assert_location_eq!(location.get_expansion_location(), f, 3, 31, 79);
        assert_location_eq!(location.get_file_location(), f, 3, 31, 79);
        assert_eq!(location.get_presumed_location(), ("presumed.h".into(), 321, 31));
        assert_location_eq!(location.get_spelling_location(), f, 3, 31, 79);
        assert!(location.is_in_main_file());
        assert!(!location.is_in_system_header());
    });

    // SourceRange _______________________________

    with_file(&clang, "int a = 322", |_, f| {
        let range = SourceRange::new(f.get_location(1, 5), f.get_location(1, 6));
        assert_location_eq!(range.get_end().get_spelling_location(), f, 1, 6, 5);
        assert_location_eq!(range.get_start().get_spelling_location(), f, 1, 5, 4);
    });

    // TranslationUnit ___________________________

    with_translation_unit(&clang, "test.c", "int a = 322;", &[], |_, f, tu| {
        assert_eq!(format!("{:?}", tu), format!("TranslationUnit {{ spelling: {:?} }}", f));
    });

    //- from_ast ---------------------------------
    //- save -------------------------------------

    with_translation_unit(&clang, "test.c", "int a = 322;", &[], |d, _, tu| {
        let file = d.join("test.c.gch");
        tu.save(&file).unwrap();
        let mut index = Index::new(&clang, false, false);
        let _ = TranslationUnit::from_ast(&mut index, &file).unwrap();
    });

    //- from_source ------------------------------

    with_temporary_file("test.c", "int a = 322;", |_, f| {
        let mut index = Index::new(&clang, false, false);
        let unsaved = &[Unsaved::new(f, "int a = 644;")];
        let options = ParseOptions::default();
        let _ = TranslationUnit::from_source(&mut index, f, &[], unsaved, options).unwrap();
    });

    //- get_file ---------------------------------

    with_translation_unit(&clang, "test.c", "int a = 322;", &[], |d, _, tu| {
        assert_eq!(tu.get_file(d.join("test.cpp")), None);
    });

    //- get_memory_usage -------------------------

    with_translation_unit(&clang, "test.c", "int a = 322;", &[], |_, _, tu| {
        let usage = tu.get_memory_usage();
        assert_eq!(usage.get(&MemoryUsage::Selectors), Some(&0));
    });

    //- reparse ----------------------------------

    with_translation_unit(&clang, "test.c", "int a = 322;", &[], |_, f, tu| {
        let _ = tu.reparse(&[Unsaved::new(f, "int a = 644;")]).unwrap();
    });
}
