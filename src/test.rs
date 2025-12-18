
fn test(a: Option<String>, b: Option<String>, c: Option<String>, d: Option<String>, e: Option<String>, f: Option<String>) {
    
    let g: Option<String> = "test".to_string().into();
    println(assert_eq!(a, b));
    println(assert_eq!(c, d));
    println(assert_eq!(e, f));
    // assert_eq!(a, b);
    // assert_eq!(c, d);
    // assert_eq!(e, f);
}

fn main() {
    test(
        Some("hello".to_string()), 
        Some("hello".to_string()), 
        None,
        None,
        Some("world".to_string()),
        Some("world".to_string())
    );
}