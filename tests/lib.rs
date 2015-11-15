extern crate clang;
extern crate uuid;

use std::env;
use std::fs;
use std::io::{Write};
use std::path::{Path};

use clang::*;

use uuid::{Uuid};

macro_rules! assert_location_eq {
    ($location:expr, $file:expr, $line:expr, $column:expr, $offset:expr) => ({
        let location = Location { file: $file, line: $line, column: $column, offset: $offset };
        assert_eq!($location, location);
    })
}

fn with_temporary_directory<F>(mut f: F) where F: FnMut(&Path) {
    let directory = env::temp_dir().join(Uuid::new_v4().to_simple_string());
    fs::create_dir(&directory).unwrap();
    f(&directory);
    fs::remove_dir_all(&directory).unwrap();
}

fn with_temporary_file<F>(name: &str, contents: &str, mut f: F) where F: FnMut(&Path, &Path) {
    with_temporary_directory(|d| {
        let file = d.join(name);
        fs::File::create(&file).unwrap().write_all(contents.as_bytes()).unwrap();
        f(d, &file);
    });
}

fn with_translation_unit<'c, F>(
    clang: &'c Clang, name: &str, contents: &str, arguments: &[&str], mut f: F
) where F: FnMut(&Path, &Path, TranslationUnit) {
    with_temporary_file(name, contents, |d, file| {
        let mut index = Index::new(clang, false, false);
        let options = ParseOptions::default();
        let tu = TranslationUnit::from_source(&mut index, file, arguments, &[], options).unwrap();
        f(d, &file, tu);
    });
}

#[test]
fn test() {
    let clang = Clang::new().unwrap();

    // File ______________________________________

    with_translation_unit(&clang, "test.h", "int a = 322;", &[], |_, f, tu| {
        let file = tu.get_file(f).unwrap();
        assert!(file.get_id() != (0, 0, 0));
        assert_eq!(file.get_location(1, 5), file.get_offset_location(4));
        assert_eq!(file.get_path(), f.to_path_buf());
        assert!(file.get_time() != 0);
        assert!(!file.is_include_guarded());

        assert_eq!(file, file);
        assert_eq!(format!("{:?}", file), format!("File {{ path: {:?} }}", f));
    });

    let source = "
        #ifndef _TEST_H_
        #define _TEST_H_
        int a = 322;
        #endif
    ";

    with_translation_unit(&clang, "test.h", source, &[], |_, f, tu| {
        assert!(tu.get_file(f).unwrap().is_include_guarded());
    });

    // Index _____________________________________

    let mut index = Index::new(&clang, false, false);

    let mut priority = BackgroundPriority { editing: false, indexing: false };
    assert_eq!(index.get_background_priority(), priority);

    priority.editing = true;
    index.set_background_priority(priority);
    assert_eq!(index.get_background_priority(), priority);

    // SourceLocation ____________________________

    let source = r#"
        #define ADD(LEFT, RIGHT) (LEFT + RIGHT)
        #line 322 "presumed.h"
        int add(int left, int right) { return ADD(left, right); }
    "#;

    with_translation_unit(&clang, "test.h", source, &[], |_, f, tu| {
        let file = tu.get_file(f).unwrap();
        let location = file.get_location(3, 51);
        assert_location_eq!(location.get_expansion_location(), file, 3, 31, 79);
        assert_location_eq!(location.get_file_location(), file, 3, 31, 79);
        assert_location_eq!(location.get_spelling_location(), file, 3, 31, 79);
        assert_eq!(location.get_presumed_location(), ("presumed.h".into(), 321, 31));
        assert!(location.is_in_main_file());
        assert!(!location.is_in_system_header());

        assert_eq!(location, location);

        let expected = format!(
            "SourceLocation {{ file: {:?}, line: {}, column: {}, offset: {} }}", file, 3, 31, 79
        );

        assert_eq!(format!("{:?}", location), expected);
    });

    // SourceRange _______________________________

    with_translation_unit(&clang, "test.h", "int a = 322;", &[], |_, f, tu| {
        let file = tu.get_file(f).unwrap();
        let start = file.get_location(1, 5);
        let end = file.get_location(1, 6);
        let range = SourceRange::new(start, end);
        assert_eq!(range.get_end(), end);
        assert_eq!(range.get_start(), start);

        assert_eq!(range, range);
        let expected = format!("SourceRange {{ start: {:?}, end: {:?} }}", start, end);
        assert_eq!(format!("{:?}", range), expected);
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

    with_translation_unit(&clang, "test.c", "int a = 322;", &[], |d, f, tu| {
        assert!(tu.get_file(f).is_some());
        assert!(tu.get_file(d.join("test.cpp")).is_none());
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
