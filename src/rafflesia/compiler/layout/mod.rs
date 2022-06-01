pub mod parser;

use std::collections::HashMap;
use anyhow::{anyhow, Context, Result};
use thiserror::Error;
use parser::View;
use swrs::api::view::{View as SWRSView, ViewType};
use swrs::parser::view::models::layout::{gravity, Orientation, Size};

/// Compiles a parsed view into an swrs [`swrs::api::view::View`].
pub fn compile_view_tree(mut parsed: View) -> Result<SWRSView> {
    fn compile(mut parsed: View, state: &mut u32) -> Result<SWRSView> {
        Ok(SWRSView {
            id: if let Some(id) = parsed.view_id { id } else {
                *state += 1;
                format!("view{}", *state - 1)
            },
            background_color: Default::default(),
            height: if let Some(hmap) = &mut parsed.attributes {
                    if let Some(height) = hmap.remove("height") {
                        match height.as_str() {
                            "match_parent" => Size::MatchParent,
                            "wrap_content" => Size::WrapContent,
                            _ => Size::Fixed(
                                height.parse()
                                    .context(format!(
                                        "Failed to convert height value `{}` to an integer", height
                                    ))?
                            )
                        }
                    } else { Size::WrapContent }
                } else { Size::WrapContent },
            width: if let Some(hmap) = &mut parsed.attributes {
                    if let Some(width) = hmap.remove("width") {
                        match width.as_str() {
                            "match_parent" => Size::MatchParent,
                            "wrap_content" => Size::WrapContent,
                            _ => Size::Fixed(
                                width.parse()
                                    .context(format!(
                                        "Failed to convert height value `{}` to an integer", height
                                    ))?
                            )
                        }
                    } else { Size::WrapContent }
                } else { Size::WrapContent },
            padding: todo!(),
            margin: todo!(),
            weight: 0,
            weight_sum: 0,
            layout_gravity: Default::default(),
            view: Ok(view_name_to_view_type(
                parsed.name,
                &mut parsed.attributes.unwrap_or_else(|| HashMap::new())
            )?),
            children: vec![],
            raw: todo!(),
            view_type: 0 // todo: fix this shayt
        })
    }

    compile(parsed, &mut 0u32)
}

fn view_name_to_view_type(name: String, attributes: &mut HashMap<String, String>)
    -> core::result::Result<ViewType, ViewNameConversionError> {

    Ok(match name.as_str() {
        "LinearLayout" => {
            ViewType::LinearLayout {
                orientation: if let Some(orientation) = attributes.remove("orientation") {
                    match orientation.as_str() {
                        "vertical" => Orientation::Vertical,
                        "horizontal" => Orientation::Horizontal,
                        _ => return Err(ViewNameConversionError::InvalidAttributeValue {
                            view_name: name,
                            attribute_name: "orientation".to_string(),
                            attribute_value: orientation,
                            possible_values: vec!["vertical".to_string(), "horizontal".to_string()]
                        })
                    }
                } else { Orientation::Vertical }, // default is vertical if no orientation specified

                gravity: if let Some(gravity) = attributes.remove("gravity") {
                    let values: Vec<&str> = gravity.split("|").map(|s| s.trim()).collect();
                    let mut result = gravity::Gravity(gravity::NONE);

                    let mut horizontal_taken = false;
                    let mut vertical_taken = false;

                    // errors when an incompatible gravity is specified
                    macro_rules! err_if_taken {
                        ($taken_var:ident,$incompatible:expr,$incompatible_with:expr) => {
                            if $taken_var {
                                return Err(ViewNameConversionError::IncompatibleAttributeValueItem {
                                    view_name: name,
                                    attribute_name: "gravity".to_string(),
                                    attribute_value: gravity,
                                    attribute_value_item_incompatible: $incompatible.to_string(),
                                    attribute_value_item_incompatible_with: $incompatible_with.to_string()
                                })
                            }

                            $taken_var = true;
                        };
                    }

                    for val in values {
                        result.0 |= match val {
                            "center_horizontal" => gravity::CENTER_HORIZONTAL,
                            "center_vertical" => gravity::CENTER_VERTICAL,
                            "center" => gravity::CENTER,
                            "left" => {
                                err_if_taken!(horizontal_taken, "left", "right");
                                gravity::LEFT
                            },
                            "right" => {
                                err_if_taken!(horizontal_taken, "right", "left");
                                gravity::RIGHT
                            },
                            "top" => {
                                err_if_taken!(vertical_taken, "top", "bottom");
                                gravity::TOP
                            },
                            "bottom" => {
                                err_if_taken!(vertical_taken, "bottom", "top");
                                gravity::BOTTOM
                            },
                            _ => return Err(ViewNameConversionError::InvalidAttributeValueItem {
                                view_name: name,
                                attribute_name: "gravity".to_string(),
                                attribute_value: gravity,
                                attribute_value_item: val.to_string(),
                                possible_value_items: vec![
                                    "center_horizontal".to_string(), "center_vertical".to_string(),
                                    "center".to_string(),
                                    "left".to_string(), "right".to_string(),
                                    "top".to_string(), "bottom".to_string()
                                ]
                            })
                        }
                    }

                    result
                } else { gravity::Gravity(gravity::NONE) }
            }
        },
        "ScrollView" => todo!(),
        "Button" => todo!(),
        "TextView" => todo!(),
        "EditText" => todo!(),
        "ImageView" => todo!(),
        "WebView" => todo!(),
        "ProgressBar" => todo!(),
        "ListView" => todo!(),
        "Spinner" => todo!(),
        "CheckBox" => todo!(),
        "Switch" => todo!(),
        "SeekBar" => todo!(),
        "CalendarView" => todo!(),
        "Switch" => todo!(),
        "Fab" => todo!(),
        "AdView" => todo!(),
        "MapView" => todo!(),
        _ => return Err(ViewNameConversionError::UnknownView { view_name: name })
    })
}

#[derive(Error, Debug)]
pub enum ViewNameConversionError {
    #[error("invalid attribute value given on attribute `{attribute_name}`: `{attribute_value}`. \
possible values: `{possible_values:?}`")]
    InvalidAttributeValue {
        view_name: String,
        attribute_name: String,
        attribute_value: String,
        possible_values: Vec<String>,
    },
    #[error("invalid attribute value item given on attribute `{attribute_name}`: \
`{attribute_value_item}`, full value: `{attribute_value}`. possible value items: \
`{possible_value_items:?}`")]
    InvalidAttributeValueItem {
        view_name: String,
        attribute_name: String,
        attribute_value: String,
        attribute_value_item: String,
        possible_value_items: Vec<String>
    },
    #[error("incompatible attribute value given on attribute \
`{attribute_name}`: `{attribute_value_item_incompatible}`, full value: `attribute_value`. \
the item `{attribute_value_item_incompatible}` is incompatible with \
`{attribute_value_item_incompatible_with}`")]
    IncompatibleAttributeValueItem {
        view_name: String,
        attribute_name: String,
        attribute_value: String,
        attribute_value_item_incompatible: String,
        attribute_value_item_incompatible_with: String,
    },
    #[error("unknown view: `{view_name}`")]
    UnknownView {
        view_name: String
    }
}