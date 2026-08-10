#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    let structure = Structure { foo: 322, bar: 6.44 };
    println!("{:?}", structure);
}
