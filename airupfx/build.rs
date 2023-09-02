#[cfg(feature = "config")]
include!("src/config/build.rs");

fn main() {
    #[cfg(feature = "config")]
    uneval::to_out_dir(
        serde_json::from_str::<BuildManifest>(include_str!("../build_manifest.json")).unwrap(),
        "build_manifest.rs",
    )
    .unwrap();
}
