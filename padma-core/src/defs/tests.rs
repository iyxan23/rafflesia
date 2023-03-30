use super::parse_defs;

#[test]
fn simple() {
    // this looks really stupid, this is just a way to see how the parser operates
    //
    // simple_logger::SimpleLogger::new()
    //     .with_level(log::LevelFilter::Trace)
    //     .with_colors(true)
    //     .init().unwrap();

    let code = r#"
b.function(s): d {
    #block(@@, @0);
}

function(b, d, s) {
    #what(#is(@0), #this(@1), #madness(@2));
}

test(s) {
    #`$escaped$%^@*!!name!?!?`(@0);
    call_to_another_func(@0, "literally");
}
"#;
    let defs = parse_defs(code).unwrap();
    println!("{:?}", defs);
}