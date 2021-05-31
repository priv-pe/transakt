use std::path::PathBuf;
use transakt::Transakt;
use transakt::transaction::ClientId;
use transakt::currency::Currency;
use std::str::FromStr;

#[test]
pub fn scenario1() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let mut filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    filepath.push("tests/scenario1.csv");

    let transakt = Transakt::read_from_csv(&filepath).unwrap();
    let accounts = transakt.get_accounts_map();
    let account = accounts.get(&ClientId::new(1)).unwrap();
    assert_eq!(account.total().unwrap(), Currency::from_str("0.8999").unwrap());
}

#[test]
pub fn scenario2() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let mut filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    filepath.push("tests/scenario2.csv");

    let transakt = Transakt::read_from_csv(&filepath).unwrap();
    let accounts = transakt.get_accounts_map();
    let account = accounts.get(&ClientId::new(1)).unwrap();
    assert_eq!(account.total().unwrap(), Currency::from_str("-1").unwrap());
}

#[test]
pub fn scenario3() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let mut filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    filepath.push("tests/scenario3.csv");

    let transakt = Transakt::read_from_csv(&filepath).unwrap();
    let accounts = transakt.get_accounts_map();
    let account = accounts.get(&ClientId::new(1)).unwrap();
    assert_eq!(account.total().unwrap(), Currency::from_str("1").unwrap());

    let account = accounts.get(&ClientId::new(2)).unwrap();
    assert_eq!(account.total().unwrap(), Currency::from_str("2").unwrap());

    let account = accounts.get(&ClientId::new(3)).unwrap();
    assert_eq!(account.total().unwrap(), Currency::from_str("3.1415").unwrap());

    let account = accounts.get(&ClientId::new(6)).unwrap();
    assert_eq!(account.total().unwrap(), Currency::from_str("42").unwrap());

    let account = accounts.get(&ClientId::new(9)).unwrap();
    assert_eq!(account.total().unwrap(), Currency::from_str("0.123").unwrap());

    let account = accounts.get(&ClientId::new(100));
    assert!(account.is_none());
}

#[test]
pub fn scenario4() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let mut filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    filepath.push("tests/scenario4.csv");

    let transakt = Transakt::read_from_csv(&filepath).unwrap();
    let accounts = transakt.get_accounts_map();
    let account = accounts.get(&ClientId::new(1)).unwrap();
    assert_eq!(account.total().unwrap(), Currency::from_str("0").unwrap());
    assert!(account.is_locked());
}
