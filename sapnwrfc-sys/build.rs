use std::{env, path::PathBuf};

fn main() {
    let sdk_home = PathBuf::from(
        env::var("SAPNWRFC_HOME").expect("Environment variable SAPNWRFC_HOME is not set"),
    );
    let lib_path = sdk_home.join("lib");
    let include_path = sdk_home.join("include");

    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-lib=dylib=sapnwrfc");
    println!("cargo:include={}", include_path.display());

    println!("cargo:rerun-if-changed=wrapper.h");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::builder()
        .header("wrapper.h")
        .allowlist_function("_*Rfc.*|.*DecFloat.*")
        .allowlist_type("_*(?:DATA|RFC|SAP|DECF).*")
        .allowlist_var("_*(?:DATA|RFC|SAP|DECF).*")
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .derive_debug(false)
        .derive_default(true)
        .clang_arg(format!("-I{}", include_path.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Couldn't generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings output");
}
