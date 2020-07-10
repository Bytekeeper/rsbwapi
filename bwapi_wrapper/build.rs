extern crate bindgen;

use regex::Regex;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    //    println!("cargo:rustc-link-lib=bz2");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .clang_arg("-xc++")
        .clang_arg("-std=c++14")
        .clang_arg("-Irepltype")
        .clang_arg("-Ibwapi/bwapi/include")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .whitelist_type(".*GameData")
        .whitelist_type(".*GameTable")
        .whitelist_type(".*Enum")
        .ignore_methods()
        .ignore_functions()
        .opaque_type("std::.*")
        .rustfmt_bindings(true)
        .derive_default(true)
        .derive_eq(true)
        .derive_hash(true)
        //        .disable_name_namespacing()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let result = bindings.to_string();
    let mut file = File::create(out_path.join("bindings.rs")).unwrap();
    let re =
        Regex::new(r"#\s*\[\s*derive\s*\((?P<d>[^)]+)\)\]\s*pub\s+enum").unwrap();
    let changed = re.replace_all(
        &result,
        "#[derive($d, FromPrimitive)]\npub enum",
    );
    assert_ne!(changed, result, "Could not add FromPrimitive to bindings!");
    file.write_all(changed.as_bytes())
        .expect("Couldn't write bindings!");
}
