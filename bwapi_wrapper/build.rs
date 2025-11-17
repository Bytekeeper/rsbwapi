use bindgen::Formatter;
use bindgen::callbacks::{ParseCallbacks, TypeKind};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
struct FromPrimitiveDerive;

impl ParseCallbacks for FromPrimitiveDerive {
    fn add_derives(&self, info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
        if info.kind == TypeKind::Enum {
            vec!["FromPrimitive".to_string()]
        } else {
            vec![]
        }
    }
}

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows" {
        panic!(
            "RSBWAPI does not yet support targeting OpenBW, please compile with a Windows target."
        );
    }
    println!("cargo::rerun-if-changed=bwapi");

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
        .allowlist_type("BWAPI::(GameTable|.*Enum|MouseButton|Key|UnitData|RegionData|GameData)")
        .allowlist_type("BWAPIC::(UnitCommand|.*Enum)")
        .ignore_methods()
        .ignore_functions()
        .opaque_type("std::.*")
        .formatter(Formatter::Rustfmt)
        // .derive_default(true)
        // .derive_eq(true)
        .derive_hash(true)
        .parse_callbacks(Box::new(FromPrimitiveDerive))
        //        .disable_name_namespacing()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // .clang_arg("--target=i686-unknown-linux-gnu")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    // todo!("Target src/");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let result = bindings.to_string();
    let mut file = File::create(out_path.join("bindings.rs")).unwrap();
    // let re = Regex::new(r"#\s*\[\s*derive\s*\((?P<d>[^)]+)\)\s*\]\s*pub\s+enum").unwrap();
    // let changed = re.replace_all(&result, "#[derive($d, FromPrimitive)]\npub enum");
    // assert_ne!(changed, result, "Could not add FromPrimitive to bindings!");
    // file.write_all(changed.as_bytes())
    // .expect("Couldn't write bindings!");
    let _ = file.write_all(result.as_bytes());
}
