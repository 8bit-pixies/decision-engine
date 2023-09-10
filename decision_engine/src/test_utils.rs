use std::path::{Path, PathBuf};

pub fn crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .unwrap()
}

pub fn testdata_dir() -> PathBuf {
    crate_dir().join("testdata")
}
