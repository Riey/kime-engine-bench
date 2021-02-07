fn main() {
    println!("cargo:rustc-link-lib=dylib=hangul");
    let bindings = bindgen::Builder::default()
        .header("./wrapper.h")
        .whitelist_function("hangul.+")
        .whitelist_type("hangul.+")
        .generate()
        .expect("Unable to generate bindings");

    bindings.write_to_file("src/hangul.rs").unwrap();
}
