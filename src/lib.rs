pub fn intro_animation() {
    let frames =
        r#"
=== RustPass ================================
Secure • Fast • Rust-Powered Password Manager
=============================================
        "#
    ;
    clear_terminal();

    println!("{frames}");
}

pub fn clear_terminal() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}
