use super::parser::parse_logic;

#[test]
fn parse_1() {
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

#[test]
fn parse_2() {
    let _ = env_logger::builder().is_test(true).try_init();

    let code = r#"
number a
list<string> s

onCreate {
    a = 10 + 3 * 2 + 2 ** 2
    func("hi", 4.4, 5)
    s.push("something")
}
"#.trim();

    let ast = parse_logic(code)
        .expect("failed to parse code");

    println!("{:?}", ast);
}

#[test]
fn parse_3() {
    let _ = env_logger::builder().is_test(true).try_init();

    let code = r#"
string b

onCreate {
    // empty
}

number a

button1.onClick {
    toast("hello world")
    returning_function().access.something[index]("call").another
}
"#.trim();

    let ast = parse_logic(code)
        .expect("failed to parse code");

    println!("{:?}", ast);
}
