use std::path::Path;

#[cfg(feature = "_internal")]
fn main() {
    println!("cargo::rerun-if-changed=../build_manifest.json");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let build_manifest: ciborium::Value =
        serde_json::from_reader(std::fs::File::open("../build_manifest.json").unwrap()).unwrap();
    let mut file = std::fs::File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(Path::new(&out_dir).join("build_manifest.cbor"))
        .unwrap();

    ciborium::into_writer(&build_manifest, &mut file).unwrap();
}

#[cfg(not(feature = "_internal"))]
fn main() {}
