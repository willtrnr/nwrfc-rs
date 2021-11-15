use std::{env, path::PathBuf};

fn main() {
    let dst_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let sdk_home = PathBuf::from(
        env::var("SAPNWRFC_HOME").expect("Environment variable SAPNWRFC_HOME is not set")
    );
    let include_path = sdk_home.join("include");
    let lib_path = sdk_home.join("lib");

    bindgen::builder()
        .header(include_path.join("sapnwrfc.h").to_string_lossy())
        .allowlist_function("_*Rfc.*|.*DecFloat.*")
        .allowlist_type("_*(?:DATA|RFC|SAP|DECF).*")
        .allowlist_var("_*(?:DATA|RFC|SAP|DECF).*")
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .derive_default(true)
        .clang_arg("-DSAPwithUNICODE")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Couldn't generate bindings")
        .write_to_file(dst_path.join("bindings.rs"))
        .expect("Couldn't write bindings output");

    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-lib=dylib=sapnwrfc");
    println!("cargo:rustc-link-lib=dylib=sapucum");
    println!("cargo:include={}", include_path.display());
}
