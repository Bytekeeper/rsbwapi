extern crate bindgen;

use regex::Regex;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    should_replace1();
    should_replace2();

    let host_target = std::env::var("HOST").unwrap();
    std::env::set_var(
        "BINDGEN_EXTRA_CLANG_ARGS",
        format!("--target={}", host_target),
    );

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .clang_arg("-xc++")
        .clang_arg("-std=c++14")
        .clang_arg("-Ibwapi/bwapi/include")
        .clang_arg("-I.")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .allowlist_type("BWAPI::.*")
        //        .whitelist_type("BWAPI::.*GameTable")
        //      .whitelist_type("BWAPI::.*Enum")
        .ignore_methods()
        .ignore_functions()
        .opaque_type("std::.*")
        .rustfmt_bindings(true)
        // .derive_default(true)
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
    let re = Regex::new(r"#\s*\[\s*derive\s*\((?P<d>[^)]+)\)\s*\]\s*pub\s+enum").unwrap();
    let changed = re.replace_all(&result, "#[derive($d, FromPrimitive)]\npub enum");
    assert_ne!(changed, result, "Could not add FromPrimitive to bindings!");
    file.write_all(changed.as_bytes())
        .expect("Couldn't write bindings!");
}

fn should_replace1() {
    // GIVEN
    let test = "# [ derive ( Debug , Copy , Clone , PartialEq , Eq , Hash ) ] pub enum std_deque__bindgen_ty_1";

    // WHEN
    let re = Regex::new(r"#\s*\[\s*derive\s*\((?P<d>[^)]+)\)\s*\]\s*pub\s+enum").unwrap();
    let changed = re.replace_all(test, "#[derive($d, FromPrimitive)]\npub enum");

    // THEN
    assert_eq!("#[derive( Debug , Copy , Clone , PartialEq , Eq , Hash , FromPrimitive)]\npub enum std_deque__bindgen_ty_1", changed);
}

fn should_replace2() {
    // GIVEN
    let test = "# [ derive ( Debug , Copy , Clone , PartialEq , Eq , Hash ) ] pub enum BWAPI_Text_Size_Enum # [ derive ( Debug , Copy , Clone , PartialEq , Eq , Hash ) ] pub enum BWAPIC_CommandType_Enum ";

    // WHEN
    let re = Regex::new(r"#\s*\[\s*derive\s*\((?P<d>[^)]+)\)\s*\]\s*pub\s+enum").unwrap();
    let changed = re.replace_all(test, "#[derive($d, FromPrimitive)]\npub enum");

    // THEN
    assert_eq!("#[derive( Debug , Copy , Clone , PartialEq , Eq , Hash , FromPrimitive)]\npub enum BWAPI_Text_Size_Enum #[derive( Debug , Copy , Clone , PartialEq , Eq , Hash , FromPrimitive)]\npub enum BWAPIC_CommandType_Enum ", changed);
}
