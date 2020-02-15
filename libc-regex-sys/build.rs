extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::builder()
        .header("wrapper.h")
        .whitelist_type("regex_t")
        .whitelist_function("regcomp")
        .whitelist_function("regexec")
        .whitelist_function("regfree")
        .whitelist_var("REG_EXTENDED")
        .generate()
        .expect("Error generating bindings for regex");

    let out_path: PathBuf = env::var("OUT_DIR").unwrap().into();

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect(&format!("Error writing bindings to {}", out_path.display()));
}
