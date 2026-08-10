extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings!")
        .write_to_file(out.join("bindings.rs"))
        .expect("Unable to write bindings!");
}
