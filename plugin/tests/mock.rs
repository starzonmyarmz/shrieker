use std::path::PathBuf;
use std::sync::Arc;

use shrieker::bridge;
use shrieker::params::ShriekerParams;
use shrieker::presets;

#[test]
fn ui_mock_matches_params() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui/mock.json");
    let mock = serde_json::to_string_pretty(&bridge::mock(Arc::new(ShriekerParams::default()), presets::json()))
        .unwrap()
        + "\n";
    if std::env::var_os("BLESS").is_some() {
        std::fs::write(&path, mock).unwrap();
        return;
    }
    let current = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(current == mock, "ui/mock.json is out of date with the params; rerun with BLESS=1");
}
