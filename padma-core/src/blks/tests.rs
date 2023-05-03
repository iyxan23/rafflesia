#[test]
fn simple() {
    let content = r#"[component]doToast: "toast %s";
[operator]`&&`(b): "%b and %b";
[operator]toString(s): "toString %d without decimal";
[operator]`+`(d): "%d + %d";
[operator]stringJoin(s): "%s join %s";
[operator]stringLength(d): "length of %d";
[math]random: "pick random %d to %d";"#;

    let blks = super::parse_blks(content).unwrap();
    println!("{:?}", blks);

    for def in blks.0 {
        println!("{:?}", def);
    }
}
