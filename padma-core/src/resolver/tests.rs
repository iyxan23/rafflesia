use super::Resolver;
use super::blks::parse_blks;
use super::defs::parse_defs;

#[test]
fn stress_test() {
    let blks = r#"
[component]doToast: "toast %s";
[operator]`&&`(b): "%b and %b";
[operator]toString(s): "toString %d without decimal";
[operator]`+`(d): "%d + %d";
[operator]stringJoin(s): "%s join %s";
[operator]stringLength(d): "length of %d";
[math]random: "pick random %d to %d";
[view]doThings: "idk";
[component]doSomething: "hmm %d";"#;

    let defs = r#"
toast(s) {
    #doThings();
    #doToast(#stringJoin("what", @0));
}

d.doSomething() {
    #doSomething(#`+`(50, @@));
}

s.concat(s): s {
    < #stringJoin(@@, @0);
}

random(d, d): d {
    < #random(@0, @1);
}
"#;

    let blks = parse_blks(blks).expect("failed to parse blks");
    let defs = parse_defs(defs).expect("failed to parse defs");

    let resolver = Resolver::new(vec![blks], vec![defs]);
    let definition = resolver.resolve().expect("failed to resolve");
    
    for (sign, def) in definition {
        println!("function {:?}", sign);
        println!("  def signature: {:?}", def.signature);
        println!("  blocks: ");

        for block in def.blocks.blocks {
            println!(" -> [{}] \"{}\" ({:?}) -> {:?}", block.opcode, block.spec.to_string(), block.arguments, block.return_type);
        }
        println!("");
    }
}