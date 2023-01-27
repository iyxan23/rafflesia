#[test]
fn simple() {
    let content = r#"// contains block definitions that'll be used on the types

[component]doToast: "toast %s"
[operator]&&(b): "%b and %b"
[operator]toString(s): "toString %d without decimal"
[operator]+(d): "%d + %d"
[operator]stringJoin(s): "%s join %s"
[operator]stringLength(d): "length of %d"
[math]random: "pick random %d to %d""#;

    super::parse(content).unwrap();
}