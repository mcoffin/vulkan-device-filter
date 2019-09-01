extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::path::PathBuf;

fn main() {
    let _vulkan = pkg_config::probe_library("vulkan")
        .expect("Error finding vulkan with pkg-config");

    let bindings = bindgen::builder()
        .header("wrapper.h")
        .whitelist_type("PFN.+")
        .whitelist_type("VkLayerInstanceCreateInfo")
        .generate()
        .expect("Error generating libobs bindings");

    let out_path: PathBuf = env::var("OUT_DIR").unwrap().into();

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect(&format!("Error writing bindings to {}", out_path.display()));
}
