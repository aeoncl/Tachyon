extern crate bindgen;
extern crate cc;

fn main() {
    cc::Build::new().files(["lib/libsiren/common.c", "lib/libsiren/rmlt.c", "lib/libsiren/dct4.c", "lib/libsiren/encoder.c", "lib/libsiren/huffman.c"]).include("lib/libsiren").compile("libsiren");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate().expect("Bindings to be generated");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    bindings.write_to_file(out_path.join("bindings.rs")).expect("binding.rs to be written");

}