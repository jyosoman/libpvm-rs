extern crate cbindgen;

use std::env;

fn generate_with_lang(crate_dir: &str, lang: cbindgen::Language, out: &str) {
    let cfg = cbindgen::Config::from_root_or_default(std::path::Path::new(crate_dir));

    cbindgen::Builder::new()
        .with_config(cfg)
        .with_language(lang)
        .with_crate(&crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out);
}

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    generate_with_lang(&crate_dir, cbindgen::Language::C, "src/include/opus.h");

    generate_with_lang(&crate_dir, cbindgen::Language::Cxx, "src/include/opus.hpp");
}