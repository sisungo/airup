use std::sync::OnceLock;

static BUILD_MANIFEST: OnceLock<serde_json::Value> = OnceLock::new();

pub fn build_manifest() -> &'static serde_json::Value {
    BUILD_MANIFEST.get_or_init(|| {
        serde_json::from_slice(include_bytes!("../../build_manifest.json"))
            .expect("bad airup build")
    })
}

pub fn runtime_dir() -> &'static str {
    build_manifest()
        .get("runtime_dir")
        .expect("bad airup build")
        .as_str()
        .expect("bad airup build")
}
