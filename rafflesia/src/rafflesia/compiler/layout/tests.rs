use super::parser::*;
use std::collections::HashMap;
use swrs::api::view::flatten_views;
use swrs::parser::Parsable;
use swrs::parser::view::models::AndroidView;
use crate::compiler::layout::compile_view_tree;

// simple DSL that constructs SWRS's View using a syntax similar to the layout's syntax
// i just hate how my is IDE complaining about an unused mut AAAHHHHHHH
macro_rules! view {
    // attr_value is ident and there is an id
    {
        $name:ident ($($attr_name:ident : $attr_value:ident),* $(,)*): $id:ident $({
            $($child:expr),* $(,)*
        })?
    } => {
        view! {
            $name ($($attr_name: stringify!($attr_value).to_string())*): $id $({
                $($child, )*
            })?
        }
    };

    // attr_value is expr and there is an id
    {
        $name:ident ($($attr_name:ident : $attr_value:expr),* $(,)*): $id:ident $({
            $($child:expr),* $(,)*
        })?
    } => {
        {
            let mut attributes = HashMap::new();
            $(attributes.insert(stringify!($attr_name).to_string(), $attr_value.to_string());)*
            let mut children = Vec::new();
            $($(children.push($child);)*)?

            View {
                name: stringify!($name).to_string(),
                attributes: if attributes.len() == 0 { None } else { Some(attributes) },
                children: if children.len() == 0 { None } else { Some(Box::new(children)) },
                view_id: Some(stringify!($id).to_string())
            }
        }
    };

    // attr_value is an ident with no view id
    {
        $name:ident ($($attr_name:ident : $attr_value:ident),* $(,)*) $({
            $($child:expr),* $(,)*
        })?
    } => {
        view! {
            $name ($($attr_name: stringify!($attr_value).to_string())*) $({
                $($child, )*
            })?
        }
    };

    // attr_value is an expr with no view id
    {
        $name:ident ($($attr_name:ident : $attr_value:expr),* $(,)*) $({
            $($child:expr),* $(,)*
        })?
    } => {
        {
            let mut attributes = HashMap::new();
            $(attributes.insert(stringify!($attr_name).to_string(), $attr_value.to_string());)*
            let mut children = Vec::new();
            $($(children.push($child);)*)?

            View {
                name: stringify!($name).to_string(),
                attributes: if attributes.len() == 0 { None } else { Some(attributes) },
                children: if children.len() == 0 { None } else { Some(Box::new(children)) },
                view_id: None
            }
        }
    };
}

#[test]
fn parser_simple() {
    let input =
        r#"LinearLayout (hello: "world") {
    TextView (text: hi): myText,

    // ignore this comment!
    TextView (
        "another": "text",
        trailing: comma,
    ),
}"#;
    let result = parse_layout(input).unwrap();
    let expected = view! {
        LinearLayout (hello: world) {
            view! { TextView (text: hi): myText },
            view! { TextView (another: "text", trailing: "comma") },
        }
    };

    // todo: assert
    assert_eq!(expected, result);
}

#[test]
fn compiler_simple() {
    let input =
        r#"LinearLayout (hello: "world") {
    TextView (text: hi): myText,

    // ignore this comment!
    TextView (
        "another": "text",
        trailing: comma,
    ),
}"#;

    let expected = r#"{"adSize":"","adUnitId":"","alpha":1.0,"checked":0,"choiceMode":0,"clickable":1,"customView":"","dividerHeight":0,"enabled":1,"firstDayOfWeek":1,"id":"view0","image":{"rotate":0,"scaleType":"CENTER"},"indeterminate":"false","index":0,"layout":{"backgroundColor":16777215,"gravity":0,"height":-1,"layoutGravity":0,"marginBottom":8,"marginLeft":8,"marginRight":8,"marginTop":8,"orientation":1,"paddingBottom":8,"paddingLeft":8,"paddingRight":8,"paddingTop":8,"weight":0,"weightSum":0,"width":-1},"max":100,"parent":"root","parentType":0,"preId":"","preIndex":0,"preParentType":0,"progress":0,"progressStyle":"","scaleX":1.0,"scaleY":1.0,"spinnerMode":1,"text":{"hint":"","hintColor":-10453621,"imeOption":0,"inputType":1,"line":0,"singleLine":0,"text":"","textColor":-16777216,"textFont":"default_font","textSize":12,"textType":0},"translationX":0.0,"translationY":0.0,"type":0}
{"adSize":"","adUnitId":"","alpha":1.0,"checked":0,"choiceMode":0,"clickable":1,"customView":"","dividerHeight":0,"enabled":1,"firstDayOfWeek":1,"id":"myText","image":{"rotate":0,"scaleType":"CENTER"},"indeterminate":"false","index":0,"layout":{"backgroundColor":16777215,"gravity":0,"height":-1,"layoutGravity":0,"marginBottom":8,"marginLeft":8,"marginRight":8,"marginTop":8,"orientation":1,"paddingBottom":8,"paddingLeft":8,"paddingRight":8,"paddingTop":8,"weight":0,"weightSum":0,"width":-1},"max":100,"parent":"view0","parentType":0,"preId":"","preIndex":0,"preParentType":0,"progress":0,"progressStyle":"","scaleX":1.0,"scaleY":1.0,"spinnerMode":1,"text":{"hint":"","hintColor":-10453621,"imeOption":0,"inputType":1,"line":0,"singleLine":0,"text":"hi","textColor":0,"textFont":"default_font","textSize":12,"textType":0},"translationX":0.0,"translationY":0.0,"type":4}
{"adSize":"","adUnitId":"","alpha":1.0,"checked":0,"choiceMode":0,"clickable":1,"customView":"","dividerHeight":0,"enabled":1,"firstDayOfWeek":1,"id":"view1","image":{"rotate":0,"scaleType":"CENTER"},"indeterminate":"false","index":0,"layout":{"backgroundColor":16777215,"gravity":0,"height":-1,"layoutGravity":0,"marginBottom":8,"marginLeft":8,"marginRight":8,"marginTop":8,"orientation":1,"paddingBottom":8,"paddingLeft":8,"paddingRight":8,"paddingTop":8,"weight":0,"weightSum":0,"width":-1},"max":100,"parent":"view0","parentType":0,"preId":"","preIndex":0,"preParentType":0,"progress":0,"progressStyle":"","scaleX":1.0,"scaleY":1.0,"spinnerMode":1,"text":{"hint":"","hintColor":-10453621,"imeOption":0,"inputType":1,"line":0,"singleLine":0,"text":"TextView","textColor":0,"textFont":"default_font","textSize":12,"textType":0},"translationX":0.0,"translationY":0.0,"type":4}"#;

    let result = parse_layout(input).unwrap();
    let result = compile_view_tree(result)
        .expect("failed to compile view");

    let result = flatten_views(vec![result], None, None)
        .into_iter()
        .try_fold(String::new(), |acc, view|
            Ok::<String, <AndroidView as Parsable>::ReconstructionError>(
                format!("{acc}\n{}", view.reconstruct()?)
            )
        )
        .expect("failed to reconstruct view");

    assert_eq!(expected, result.trim());
}