extern crate rustc_version;

fn main() {
    if rustc_version::version_matches("1.5.*") {
        println!("cargo:rustc-cfg={}", "rustc_1_5");
    }
}
