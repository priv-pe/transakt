use std::path::Path;
use transakt::Transakt;

fn main() {
    env_logger::init();
    let filename = std::env::args()
        .nth(1)
        .expect("Usage: cargo run -- <input_file>");
    let filepath = Path::new(&filename);
    Transakt::read_from_csv(filepath).unwrap();
}
