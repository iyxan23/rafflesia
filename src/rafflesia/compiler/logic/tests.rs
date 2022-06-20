use super::parser::parse_logic;

#[test]
fn simple() {
    let _ = env_logger::builder().is_test(true).try_init();

    let code = r#"
// ignore me
number a

onCreate {
    a = 10
}
"#.trim();

    let ast = parse_logic(code)
        .expect("failed to parse code");

    println!("{:?}", ast);
}