# CourseHKU
A library to deal with HKU course data.

## Add to your project
Add this to your `Cargo.toml`:
```bash
cargo add coursehku
```

## Usage
```rust
use CourseHKU::course::CourseTable;
use std::path::PathBuf;

fn main() {
    let table = CourseTable::load(PathBuf::from("data.csv"));

    let table = table
        .to_lazy()  // convert to lazy table
        .semester(1)  // filter semester 1
        .contains(&["COMP", "MATH", "ENGG"])  // filter courses by code
        .no_conflict_with(table.get_course("COMP1117").unwrap())  // filter courses that do not conflict with COMP1117
        .collect()  // collect the lazy table
        .unwrap();
    println!("{}", table);
}
```

## Run example
```bash
git clone https://github.com/adlsdztony/CourseHKU.git
cd CourseHKU
cargo run --example scheduler
```