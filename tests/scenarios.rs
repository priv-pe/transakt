use transakt::run;
use std::path::PathBuf;

#[test]
pub fn scenario1() {
    let mut filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    filepath.push("tests/scenario1.csv");

    run(&filepath);
}