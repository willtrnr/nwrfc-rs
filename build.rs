use std::{env, path::PathBuf};

fn main() {
    let sdk_home = PathBuf::from(
        env::var("SAPNWRFC_HOME").expect("Environment variable SAPNWRFC_HOME is not set"),
    );

    println!(
        "cargo:rustc-flags=-L{}",
        sdk_home.join("lib").to_string_lossy()
    );
    println!("cargo:rustc-link-lib=dylib=sapnwrfc");
    println!("cargo:rustc-link-lib=dylib=sapucum");

    println!("cargo:rerun-if-changed=wrapper.h");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::builder()
        .header("wrapper.h")
        .allowlist_function("_*Rfc.*")
        .allowlist_type("_*(?:DATA|RFC|SAP|DecFloat).*")
        .allowlist_var("_*(?:DATA|RFC|SAP|DecFloat).*")
        .allowlist_recursively(false)
        .derive_debug(false)
        .derive_default(true)
        .clang_arg(format!("-I{}", sdk_home.join("include").to_string_lossy()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Couldn't generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings output");
}
