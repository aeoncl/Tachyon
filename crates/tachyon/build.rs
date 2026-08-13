fn main() {

    let sources = [
        "common.c",
        "rmlt.c",
        "dct4.c",
        "encoder.c",
        "huffman.c",
    ];

    let root = std::path::Path::new("../../lib/libsiren");

    for source in sources {
        println!("cargo:rerun-if-changed={}", root.join(source).display());
    }

    cc::Build::new()
        .files(sources.map(|source| root.join(source)))
        .include(root)
        .warnings(false)
        .compile("siren");
}
