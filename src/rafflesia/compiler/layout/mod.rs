pub mod parser;

use std::collections::HashMap;
use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;
use thiserror::Error;
use parser::View;
use swrs::api::view::{SidesValue, View as SWRSView, ViewType};
use swrs::color::Color;
use swrs::parser::view::models::{AndroidView, SpinnerMode};
use swrs::parser::view::models::image::ImageScaleType;
use swrs::parser::view::models::layout::{gravity, Orientation, Size};
use swrs::parser::view::models::layout::gravity::Gravity;
use swrs::parser::view::models::text::{ImeOption, InputType, TextType};
use crate::compiler::layout::attr_parser::{parse_color, parse_gravity, parse_text_style};

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
                    parse_color(&*color, "background_color")?
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
                layout_gravity: if let Some(layout_gravity) = attrs.remove("layout_gravity") {
                    parse_gravity(&*layout_gravity, "layout_gravity")?
                } else { Gravity::default() },
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
        "LinearLayout" =>
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
                    parse_gravity(&*gravity, "gravity")?
                } else { Gravity(gravity::NONE) }
            },
        "ScrollView" =>
            ViewType::ScrollView {
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
                    parse_gravity(&*gravity, "gravity")?
                } else { Gravity(gravity::NONE) }
            },
        "Button" =>
            ViewType::Button {
                text: attributes.remove("text").unwrap_or_else(|| "Button".to_string()),

                text_color: if let Some(text_color) = attributes.remove("text_color") {
                    parse_color(&*text_color, "text_color")?
                } else { Color::from(0x000000) },

                text_size: if let Some(text_size) = attributes.remove("text_size") {
                    text_size.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "text_size".to_string(),
                                attribute_value: text_size,
                                err
                            }
                        ))?
                } else { 12 },

                text_style: if let Some(text_style) = attributes.remove("text_style") {
                    parse_text_style(&*text_style, "text_style")?
                } else { TextType::Normal }
            },
        "TextView" =>
            ViewType::TextView {
                text: attributes.remove("text").unwrap_or_else(|| "TextView".to_string()),

                text_color: if let Some(text_color) = attributes.remove("text_color") {
                    parse_color(&*text_color, "text_color")?
                } else { Color::from(0x000000) },

                text_size: if let Some(text_size) = attributes.remove("text_size") {
                    text_size.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "text_size".to_string(),
                                attribute_value: text_size,
                                err
                            }
                        ))?
                } else { 12 },

                single_line: if let Some(single_line) = attributes.remove("single_line") {
                    single_line.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidBoolValue {
                                attribute_name: "single_line".to_string(),
                                attribute_value: single_line,
                                err
                            }
                        ))?
                } else { false },

                // todo: validation with the resources defined in manifest soon
                text_font: attributes.remove("text").unwrap_or_else(|| "default_font".to_string()),

                text_style: if let Some(text_style) = attributes.remove("text_style") {
                    parse_text_style(&*text_style, "text_style")?
                } else { TextType::Normal },

                lines: if let Some(lines) = attributes.remove("lines") {
                    lines.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "lines".to_string(),
                                attribute_value: lines,
                                err
                            }
                        ))?
                } else { 0 }
            },
        "EditText" =>
            ViewType::EditText {
                text: attributes.remove("text").unwrap_or_else(|| "EditText".to_string()),

                text_color: if let Some(text_color) = attributes.remove("text_color") {
                    parse_color(&*text_color, "text_color")?
                } else { Color::from(0x000000) },

                text_size: if let Some(text_size) = attributes.remove("text_size") {
                    text_size.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "text_size".to_string(),
                                attribute_value: text_size,
                                err
                            }
                        ))?
                } else { 12 },

                single_line: if let Some(single_line) = attributes.remove("single_line") {
                    single_line.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidBoolValue {
                                attribute_name: "single_line".to_string(),
                                attribute_value: single_line,
                                err
                            }
                        ))?
                } else { false },

                // todo: validation with the resources defined in manifest soon
                text_font: attributes.remove("text").unwrap_or_else(|| "default_font".to_string()),

                text_style: if let Some(text_style) = attributes.remove("text_style") {
                    parse_text_style(&*text_style, "text_style")?
                } else { TextType::Normal },

                lines: if let Some(lines) = attributes.remove("lines") {
                    lines.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "lines".to_string(),
                                attribute_value: lines,
                                err
                            }
                        ))?
                } else { 0 },

                hint: attributes.remove("hint").unwrap_or_else(|| String::new()),

                hint_color: if let Some(hint_color) = attributes.remove("hint_color") {
                    parse_color(&*hint_color, "hint_color")?
                } else { Color::from(0x607d8b) }, // #607d8b

                ime_option: if let Some(ime_option) = attributes.remove("ime_option") {
                    match ime_option.as_str() {
                        "normal" => ImeOption::Normal,
                        "none" => ImeOption::None,
                        "go" => ImeOption::Go,
                        "search" => ImeOption::Search,
                        "send" => ImeOption::Send,
                        "next" => ImeOption::Next,
                        "done" => ImeOption::Done,
                        _ => return Err(ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidAttributeValue {
                                attribute_name: "ime_option".to_string(),
                                attribute_value: ime_option,
                                possible_values: vec![
                                    "normal".to_string(), "none".to_string(), "go".to_string(),
                                    "search".to_string(), "send".to_string(), "next".to_string(),
                                    "done".to_string()
                                ]
                            }
                        ))
                    }
                } else { ImeOption::Normal },

                // see docs/notes.md#Full InputType support?
                input_type: if let Some(input_type) = attributes.remove("input_type") {
                    // yes i am too lazy to do `|` shayts, it's only used in one item anyway
                    match input_type.as_str() {
                        "decimal" => InputType::NumberDecimal,
                        "signed" => InputType::NumberSigned,
                        "decimal_signed" => InputType::NumberSignedDecimal,
                        "text" => InputType::Text,
                        "password" => InputType::Password,
                        "phone" => InputType::Phone,
                        _ => return Err(ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidAttributeValue {
                                attribute_name: "input_type".to_string(),
                                attribute_value: input_type,
                                possible_values: vec![
                                    "decimal".to_string(), "signed".to_string(),
                                    "decimal_signed".to_string(), "text".to_string(),
                                    "password".to_string(), "phone".to_string()
                                ]
                            }
                        ))
                    }
                } else { InputType::Text }
            },
        "ImageView" =>
            ViewType::ImageView {
                // todo: validation with the resources defined in manifest soon
                image_res_name: attributes.remove("images").unwrap_or_else(|| String::new()),
                image_scale_type: if let Some(scale_type) = attributes.remove("scale_type") {
                    match scale_type.as_str() {
                        "center" => ImageScaleType::Center,
                        "fit_xy" => ImageScaleType::FitXy,
                        "fit_start" => ImageScaleType::FitStart,
                        "fit_end" => ImageScaleType::FitEnd,
                        "center_crop" => ImageScaleType::CenterCrop,
                        "center_inside" => ImageScaleType::CenterInside,
                        _ => return Err(ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidAttributeValue {
                                attribute_name: "scale_type".to_string(),
                                attribute_value: scale_type,
                                possible_values: vec![
                                    "center".to_string(), "fit_xy".to_string(),
                                    "fit_start".to_string(), "fit_end".to_string(),
                                    "center_crop".to_string(), "center_inside".to_string()
                                ]
                            }
                        ))
                    }
                } else { ImageScaleType::Center }
            },
        "WebView" => ViewType::WebView, // literally
        "ProgressBar" =>
            ViewType::ProgressBar {
                max_progress: if let Some(max_progress) = attributes.remove("max_progress") {
                    max_progress.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "max_progress".to_string(),
                                attribute_value: max_progress,
                                err
                            }
                        ))?
                } else { 100 },

                progress: if let Some(progress) = attributes.remove("progress") {
                    progress.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "progress".to_string(),
                                attribute_value: progress,
                                err
                            }
                        ))?
                } else { 0 },

                indeterminate: if let Some(indeterminate) = attributes.remove("indeterminate") {
                    indeterminate.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidBoolValue {
                                attribute_name: "indeterminate".to_string(),
                                attribute_value: indeterminate,
                                err
                            }
                        ))?
                } else { false },

                progress_style: if let Some(progress_style) = attributes.remove("progress_style") {
                    match progress_style.as_str() {
                        "horizontal" => "?android:progressBarStyleHorizontal",
                        "circular" | "circle" => "?android:progressBarStyle",
                        _ => return Err(ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidAttributeValue {
                                attribute_name: "progress_style".to_string(),
                                attribute_value: progress_style,
                                possible_values: vec![
                                    "horizontal".to_string(), "circular".to_string(), "circle".to_string()
                                ]
                            }
                        ))
                    }
                } else { "?android:progressBarStyle" }.to_string()
            },
        "ListView" =>
            ViewType::ListView {
                divider_height: if let Some(divider_height) = attributes.remove("divider_height") {
                    divider_height.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "divider_height".to_string(),
                                attribute_value: divider_height,
                                err
                            }
                        ))?
                } else { 0 },

                // todo: validation with the resources defined in manifest soon
                custom_view: attributes.remove("custom_view").unwrap_or_else(|| String::new())
            },
        "Spinner" =>
            ViewType::Spinner {
                spinner_mode: if let Some(spinner_mode) = attributes.remove("spinner_mode") {
                    match spinner_mode.as_str() {
                        "dropdown" => SpinnerMode::Dropdown,
                        "dialog" => SpinnerMode::Dialog,
                        _ => return Err(ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidAttributeValue {
                                attribute_name: "spinner_mode".to_string(),
                                attribute_value: spinner_mode,
                                possible_values: vec!["dropdown".to_string(), "dialog".to_string()]
                            }
                        ))
                    }
                } else { SpinnerMode::Dropdown }
            },
        "CheckBox" =>
            ViewType::CheckBox {
                checked: if let Some(checked) = attributes.remove("checked") {
                    checked.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidBoolValue {
                                attribute_name: "checked".to_string(),
                                attribute_value: checked,
                                err
                            }
                        ))?
                } else { false },

                text: attributes.remove("text").unwrap_or_else(|| "CheckBox".to_string()),

                text_color: if let Some(text_color) = attributes.remove("text_color") {
                    parse_color(&*text_color, "text_color")?
                } else { Color::from(0x000000) },

                text_size: if let Some(text_size) = attributes.remove("text_size") {
                    text_size.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "text_size".to_string(),
                                attribute_value: text_size,
                                err
                            }
                        ))?
                } else { 12 },

                text_font: attributes.remove("text").unwrap_or_else(|| "default_font".to_string()),

                text_style: if let Some(text_style) = attributes.remove("text_style") {
                    parse_text_style(&*text_style, "text_style")?
                } else { TextType::Normal },
            },
        "Switch" =>
            ViewType::Switch {
                checked: if let Some(checked) = attributes.remove("checked") {
                    checked.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidBoolValue {
                                attribute_name: "checked".to_string(),
                                attribute_value: checked,
                                err
                            }
                        ))?
                } else { false },

                text: attributes.remove("text").unwrap_or_else(|| "Switch".to_string()),

                text_color: if let Some(text_color) = attributes.remove("text_color") {
                    parse_color(&*text_color, "text_color")?
                } else { Color::from(0x000000) },

                text_size: if let Some(text_size) = attributes.remove("text_size") {
                    text_size.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "text_size".to_string(),
                                attribute_value: text_size,
                                err
                            }
                        ))?
                } else { 12 },

                text_font: attributes.remove("text").unwrap_or_else(|| "default_font".to_string()),

                text_style: if let Some(text_style) = attributes.remove("text_style") {
                    parse_text_style(&*text_style, "text_style")?
                } else { TextType::Normal },
            },
        "SeekBar" =>
            ViewType::SeekBar {
                max_progress: if let Some(max_progress) = attributes.remove("max_progress") {
                    max_progress.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "max_progress".to_string(),
                                attribute_value: max_progress,
                                err
                            }
                        ))?
                } else { 100 },

                progress: if let Some(progress) = attributes.remove("progress") {
                    progress.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "progress".to_string(),
                                attribute_value: progress,
                                err
                            }
                        ))?
                } else { 0 },
            },
        "CalendarView" =>
            ViewType::CalendarView {
                first_day_of_week: if let Some(first_day_of_the_week) = attributes.remove("first_day_of_the_week") {
                    first_day_of_the_week.parse()
                        .map_err(|err| ViewCompileError::AttributeParseError(
                            AttributeParseError::InvalidIntValue {
                                attribute_name: "first_day_of_the_week".to_string(),
                                attribute_value: first_day_of_the_week,
                                err
                            }
                        ))?
                } else { 1 }
            },
        // todo: make this illegal to be placed in regular layout, must be placed in a special place
        //       or something
        "FloatingActionButton" =>
            ViewType::Fab {
                // todo: validate with resources defined in manifest soon
                image_res_name: attributes.remove("image").unwrap_or_else(|| "".to_string())
            },
        "AdView" =>
            ViewType::AdView {
                // i have no idea what this is for, i dont use adviews
                adview_size: attributes.remove("adview_size").unwrap_or_else(|| "".to_string())
            },
        "MapView" => ViewType::MapView,
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
    #[error("invalid attribute boolean given on attribute `{attribute_name}`: `{attribute_value}`. value can only be `true` or `false`")]
    InvalidBoolValue {
        attribute_name: String,
        attribute_value: String,
        err: ParseBoolError
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

mod attr_parser {
    use swrs::color::Color;
    use swrs::parser::view::models::layout::gravity;
    use swrs::parser::view::models::layout::gravity::Gravity;
    use swrs::parser::view::models::text::TextType;
    use super::ViewCompileError;
    use super::AttributeParseError;

    pub fn parse_gravity(gravity: &str, attr_name: &str) -> Result<Gravity, ViewCompileError> {
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
                            attribute_name: attr_name.to_string(),
                            attribute_value: gravity.to_string(),
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
               other => return Err(ViewCompileError::AttributeParseError(
                   AttributeParseError::InvalidAttributeValueItem {
                       attribute_name: "gravity".to_string(),
                       attribute_value: gravity.to_string(),
                       attribute_value_item: other.to_string(),
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

       Ok(result)
   }

    pub fn parse_color(color: &str, attr_name: &str) -> Result<Color, ViewCompileError> {
        // supports "ffffff" "#ffffff" "ffffffff" "#ffffffff"
        if color.len() < 6 || color.len() > 9 {
            return Err(ViewCompileError::AttributeParseError(
                AttributeParseError::InvalidColorValue {
                    attribute_name: attr_name.to_string(),
                    attribute_value: color.to_string()
                }
            ));
        }

        Ok(Color::parse_hex(if color.len() % 2 == 0 {
            // this doesn't have a #
            &*color
        } else {
            // this does have a # at the start, check it
            if &color.chars().nth(0).unwrap() != &'#' {
                // what this doesn't start with `#`!?
                return Err(ViewCompileError::AttributeParseError(
                    AttributeParseError::InvalidColorValue {
                        attribute_name: attr_name.to_string(),
                        attribute_value: color.to_string()
                    }
                ));
            }

            &color[1..]
        }).map_err(|_| ViewCompileError::AttributeParseError(
            AttributeParseError::InvalidColorValue {
                attribute_name: attr_name.to_string(),
                attribute_value: color.to_string()
            }
        ))?)
    }

    pub fn parse_text_style(text_style: &str, attr_name: &str) -> Result<TextType, ViewCompileError> {
        let values: Vec<&str> =
            text_style.split("|").map(|s| s.trim()).collect();

        let mut bold = false;
        let mut italic = false;

        for value in values {
            match value {
                "bold" => bold = true,
                "italic" => italic = true,
                other => return Err(ViewCompileError::AttributeParseError(
                    AttributeParseError::InvalidAttributeValueItem {
                        attribute_name: attr_name.to_string(),
                        attribute_value: text_style.to_owned(),
                        attribute_value_item: other.to_string(),
                        possible_value_items: vec![
                            "bold".to_string(), "italic".to_string()
                        ]
                    }
                ))
            }
        }

        Ok(if bold && italic {
            TextType::BoldItalic
        } else if bold {
            TextType::Bold
        } else if italic {
            TextType::Italic
        } else {
            TextType::Normal
        })
    }
}