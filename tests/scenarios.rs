use std::path::PathBuf;
use transakt::run;

#[test]
pub fn scenario1() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let mut filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    filepath.push("tests/scenario1.csv");

    run(&filepath).unwrap()
}
