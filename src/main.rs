use std::path::Path;
use transakt::run;

fn main() {
    let filename = std::env::args()
        .nth(1)
        .expect("Usage: cargo run -- <input_file>");
    let filepath = Path::new(&filename);
    run(filepath).unwrap();
}
