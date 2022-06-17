pub mod parser;

use std::collections::HashMap;
use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error;
use parser::View;
use swrs::api::view::{SidesValue, View as SWRSView, ViewType};
use swrs::color::Color;
use swrs::parser::view::models::AndroidView;
use swrs::parser::view::models::layout::{gravity, Orientation, Size};
use swrs::parser::view::models::layout::gravity::Gravity;

/// Compiles a parsed view into an swrs [`swrs::api::view::View`].
pub fn compile_view_tree(parsed: View) -> Result<SWRSView, ViewCompileError> {

    fn compile(parsed: View, parent_id: &str, parent_type: i8, state: &mut u32) -> Result<SWRSView, ViewCompileError> {
        let view_id = if let Some(id) = parsed.view_id { id } else {
            *state += 1;
            format!("view{}", *state - 1)
        };

        Ok(if let Some(mut attrs) = parsed.attributes {
            let view = map_view_name_attrs(parsed.name, &mut attrs)?;

            macro_rules! attr_number_get {
                ($name:expr,$default:expr) => {
                    if let Some(val) = attrs.remove($name) {
                        val.parse()
                            .map_err(|err| ViewCompileError::AttributeParseError(
                                AttributeParseError::InvalidIntValue {
                                    attribute_name: $name.to_string(),
                                    attribute_value: val,
                                    err
                                }
                            ))?
                    } else { $default }
                };
            }

            let padding = attrs
                .remove("padding")
                .map(|e| {
                    match e.parse() {
                        Ok(val) => Ok(SidesValue {
                            top: val, right: val, bottom: val, left: val
                        }),
                        Err(err) => Err(ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "margin".to_string(),
                                attribute_value: e,
                                err
                            })
                        )
                    }
                })
                .unwrap_or_else(|| {
                    Ok(SidesValue {
                        top: attr_number_get!("padding_top", 8),
                        right: attr_number_get!("padding_right", 8),
                        bottom: attr_number_get!("padding_bottom", 8),
                        left: attr_number_get!("padding_left", 8),
                    })
                })?;

            let margin = attrs
                .remove("margin")
                .map(|e| {
                    match e.parse() {
                        Ok(val) => Ok(SidesValue {
                            top: val, right: val, bottom: val, left: val
                        }),
                        Err(err) => Err(ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "padding".to_string(),
                                attribute_value: e,
                                err
                            })
                        )
                    }
                })
                .unwrap_or_else(|| {
                    Ok(SidesValue {
                        top: attr_number_get!("margin_top", 8),
                        right: attr_number_get!("margin_right", 8),
                        bottom: attr_number_get!("margin_bottom", 8),
                        left: attr_number_get!("margin_left", 8),
                    })
                })?;

            SWRSView {
                background_color: if let Some(color) = attrs.remove("background_color") {
                    // supports "ffffff" "#ffffff" "ffffffff" "#ffffffff"
                    if color.len() < 6 || color.len() > 9 {
                        return Err(ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidColorValue {
                                attribute_name: "background_color".to_string(),
                                attribute_value: color
                            }
                        ));
                    }

                    Color::parse_hex(if color.len() % 2 == 0 {
                        // this doesn't have a #
                        &*color
                    } else {
                        // this does have a # at the start
                        &color[1..]
                    }).map_err(|_| ViewCompileError::AttributeParseError(
                        AttributeParseError::InvalidColorValue {
                            attribute_name: "background_color".to_string(),
                            attribute_value: color
                        }
                    ))?
                } else { Color::from(0xFFFFFF) },
                height: if let Some(height) = attrs.remove("height") {
                    match height.as_str() {
                        "match_parent" => Size::MatchParent,
                        "wrap_content" => Size::WrapContent,
                        _ => Size::Fixed(
                            height.parse()
                                .map_err(|err| ViewCompileError::AttributeParseError(
                                    AttributeParseError::InvalidIntValue {
                                        attribute_name: "height".to_string(),
                                        attribute_value: height,
                                        err
                                    }
                                ))?
                        )
                    }
                } else { Size::WrapContent },
                width: if let Some(width) = attrs.remove("width") {
                    match width.as_str() {
                        "match_parent" => Size::MatchParent,
                        "wrap_content" => Size::WrapContent,
                        _ => Size::Fixed(
                            width.parse()
                                .map_err(|err| ViewCompileError::AttributeParseError(
                                    AttributeParseError::InvalidIntValue {
                                        attribute_name: "width".to_string(),
                                        attribute_value: width,
                                        err
                                    }
                                ))?
                        )
                    }
                } else { Size::WrapContent },
                padding,
                margin,
                weight: attr_number_get!("weight", 0),
                weight_sum: attr_number_get!("weight_sum", 0),
                layout_gravity: Default::default(),
                children: vec![],
                raw: AndroidView::new_empty(view_id.as_str(), view.get_type_id(), parent_id, parent_type),
                id: view_id,
                view: Ok(view),
            }
        } else {
            // when there's no attributes provided
            let view = map_view_name_attrs(parsed.name, &mut HashMap::new())?;

            SWRSView {
                id: view_id.to_string(),
                background_color: Color::from(0xffffff),
                height: Size::WrapContent,
                width: Size::WrapContent,
                padding: SidesValue { top: 8, right: 8, bottom: 8, left: 8 },
                margin: SidesValue { top: 8, right: 8, bottom: 8, left: 8},
                weight: 0,
                weight_sum: 0,
                layout_gravity: Default::default(),
                children: vec![],
                raw: AndroidView::new_empty(view_id.as_str(), view.get_type_id(), parent_id, parent_type),
                view: Ok(view),
            }
        })
    }

    // the root parent id of sketchware is "root"
    compile(parsed, "root", 0, &mut 0u32)
}

#[derive(Debug, Error)]
pub enum ViewCompileError {
    #[error("unknown view: `{view_name}`")]
    UnknownView {
        view_name: String
    },

    #[error("error on attribute parsing: {0}")]
    AttributeParseError(AttributeParseError),
}

/// This function maps attributes depending on the view name into the enum [`ViewType`].
///
/// The enum contains view-name-specific attributes, for instance: text for TextView, checked for
/// CheckBoxes
fn map_view_name_attrs(name: String, attributes: &mut HashMap<String, String>)
    -> Result<ViewType, ViewCompileError> {

    Ok(match name.as_str() {
        "LinearLayout" => {
            ViewType::LinearLayout {
                orientation: if let Some(orientation) = attributes.remove("orientation") {
                    match orientation.as_str() {
                        "vertical" => Orientation::Vertical,
                        "horizontal" => Orientation::Horizontal,
                        _ => return Err(ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidAttributeValue {
                                attribute_name: "orientation".to_string(),
                                attribute_value: orientation,
                                possible_values: vec!["vertical".to_string(), "horizontal".to_string()]
                            })
                        )
                    }
                } else { Orientation::Vertical }, // default is vertical if no orientation specified

                gravity: if let Some(gravity) = attributes.remove("gravity") {
                    let values: Vec<&str> = gravity.split("|").map(|s| s.trim()).collect();
                    let mut result = Gravity(gravity::NONE);

                    let mut horizontal_taken = false;
                    let mut vertical_taken = false;

                    // errors when an incompatible gravity is specified
                    macro_rules! err_if_taken {
                        ($taken_var:ident,$incompatible:expr,$incompatible_with:expr) => {
                            if $taken_var {
                                return Err(ViewCompileError::AttributeParseError(
                                    AttributeParseError::IncompatibleAttributeValueItem {
                                        attribute_name: "gravity".to_string(),
                                        attribute_value: gravity,
                                        attribute_value_item_incompatible: $incompatible.to_string(),
                                        attribute_value_item_incompatible_with: $incompatible_with.to_string()
                                    }
                                ))
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
                            _ => return Err(ViewCompileError::AttributeParseError(
                                AttributeParseError::InvalidAttributeValueItem {
                                    attribute_name: "gravity".to_string(),
                                    attribute_value: gravity.to_string(),
                                    attribute_value_item: val.to_string(),
                                    possible_value_items: vec![
                                        "center_horizontal".to_string(), "center_vertical".to_string(),
                                        "center".to_string(),
                                        "left".to_string(), "right".to_string(),
                                        "top".to_string(), "bottom".to_string()
                                    ]
                                }
                            ))
                        }
                    }

                    result
                } else { Gravity(gravity::NONE) }
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
        _ => return Err(ViewCompileError::UnknownView { view_name: name })
    })
}

#[derive(Error, Debug)]
pub enum AttributeParseError {
    #[error("invalid attribute color value given on attribute `{attribute_name}`: `{attribute_value}`. \
only supports in: ffffff, #ffffff, ffffffff, #ffffffff")]
    InvalidColorValue {
        attribute_name: String,
        attribute_value: String,
    },
    #[error("invalid attribute int value given on attribute `{attribute_name}`: `{attribute_value}`. error: {err}")]
    InvalidIntValue {
        attribute_name: String,
        attribute_value: String,
        err: ParseIntError
    },
    #[error("invalid attribute float/number value given on attribute `{attribute_name}`: `{attribute_value}`. error: {err}")]
    InvalidFloatValue {
        attribute_name: String,
        attribute_value: String,
        err: ParseFloatError
    },
    #[error("invalid attribute value given on attribute `{attribute_name}`: `{attribute_value}`. \
possible values: `{possible_values:?}`")]
    InvalidAttributeValue {
        attribute_name: String,
        attribute_value: String,
        possible_values: Vec<String>,
    },
    #[error("invalid attribute value item given on attribute `{attribute_name}`: \
`{attribute_value_item}`, full value: `{attribute_value}`. possible value items: \
`{possible_value_items:?}`")]
    InvalidAttributeValueItem {
        attribute_name: String,
        attribute_value: String,
        attribute_value_item: String,
        possible_value_items: Vec<String>
    },
    #[error("incompatible attribute value given on attribute \
`{attribute_name}`: `{attribute_value_item_incompatible}`, full value: `{attribute_value}`. \
the item `{attribute_value_item_incompatible}` is incompatible with \
`{attribute_value_item_incompatible_with}`")]
    IncompatibleAttributeValueItem {
        attribute_name: String,
        attribute_value: String,
        attribute_value_item_incompatible: String,
        attribute_value_item_incompatible_with: String,
    },
}