extern crate cbindgen;

use std::env;
use std::path::Path;

use cbindgen::Config;

fn main() {
    // this is the out dir of the iceoryx2-lang C crate not the workspace out dir,
    // therefore we need to traverse to a known location and create the path for the header
    let out_dir = env::var("OUT_DIR").expect("Target output directory");

    let mut header_path = Path::new(&out_dir)
        .join("../../../")
        .canonicalize()
        .expect("Path to iceoryx2 base dir for header generation");
    header_path.push("iceoryx2_lang_c_cbindgen/include/iox2/iceoryx2.h");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("Cargo manifest dir");

    let mut config = Config::default();
    config.language = cbindgen::Language::C;

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .expect("Unable to generate c bindings")
        .write_to_file(header_path);
}
