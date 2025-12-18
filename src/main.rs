use clap::Parser;
use serde_json::Value; //imports value type (represents json data)
use std::fs; //imports rusts file system module

fn main() {
    let string_from_json =
        fs::read_to_string("src/passwords_file.json").expect("could not read file");
    let json_data: Value = serde_json::from_str(&string_from_json).expect("invalid json");

    println!("{json_data}");

    test(
        Some("hello".to_string()),
        Some("hello".to_string()),
        None,
        None,
        Some("world".to_string()),
        Some("world".to_string()),
    );
}

fn test(
    a: Option<String>,
    b: Option<String>,
    c: Option<String>,
    d: Option<String>,
    e: Option<String>,
    f: Option<String>,
) {
    let g: Option<String> = "test".to_string().into();
    assert_eq!(a, b);
    assert_eq!(c, d);
    assert_eq!(e, f);
    // assert_eq!(a, b);
    // assert_eq!(c, d);
    // assert_eq!(e, f);
}
