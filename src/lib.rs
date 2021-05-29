use std::path::Path;

pub fn run(filepath: &Path) {
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(filepath)
        .expect("Cannot open input file");
    for record in csv.records() {
        let _transaction = record.expect("Could not parse file");
    }
}