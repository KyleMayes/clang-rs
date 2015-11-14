extern crate clang;
extern crate uuid;

use std::env;
use std::fs;
use std::io::{Write};
use std::path::{Path};

use clang::*;

use uuid::{Uuid};

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

    // Index _____________________________________

    let mut index = Index::new(&clang, false, false);

    let mut priority = BackgroundPriority { editing: false, indexing: false };
    assert_eq!(index.get_background_priority(), priority);

    priority.editing = true;
    index.set_background_priority(priority);
    assert_eq!(index.get_background_priority(), priority);

    // TranslationUnit ___________________________

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

    //- reparse ----------------------------------

    with_translation_unit(&clang, "test.c", "int a = 322;", &[], |_, f, tu| {
        let _ = tu.reparse(&[Unsaved::new(f, "int a = 644;")]).unwrap();
    });
}
