Each layout file represents a layout for an activity. The activity it bounded to it is defined in the `swproj.toml`
project configuration file. Its filename contributes nothing to the activity name.

```toml
[activity.main] # <- the name used for the activity
logic = "main.logic"
layout = "main.layout" # references to src/main.layout
```

## Layout file

Here's a simple example of a layout file that represents a `Button` centered inside a `LinearLayout`.

```text
// hello rafflesia layout

LinearLayout (
    orientation: vertical,
    layout_width: match_parent,
    layout_height: match_parent,
    gravity: center
) {
    TextView (text: "Hello world"),
    Button (text: "Click me"): myButton
}
```

A view is defined by its name, then optionally a list of attributes inside parentheses `()`, followed by a curly brace
block `{}` in which other views can be defined as its children; each child are separated with `,` and at the end, a
colon `:` can be used to define the view id for the view.

```text
ParentView (attribute: value) {
    // a children of view `ParentView` set with id of `viewId1`
    // and its attributes defined in the parentheses
    ChildrenView (otherAttribute: "string text"): viewId1,

    // children with no attributes set, but id set to `viewId2`
    AnotherChildren: viewId2,

    // children with nothing set
    PlainChildren, // trailing comma
}
```

The attributes and children can either be separated into lines or one-lined as long as they are separated using commas
(`,`).

```text
LinearLayout (
    orientation: vertical,
    layout_width: match_parent,
    layout_height: match_parent, // <- trailing comma is okay
) {
    TextView (text: "hello world"),
    TextView (text: "another text"), // <- trailing comma is okay
}
```

### Global view access

Global view access is a connection between the layout bound to an activity with its logic. It allows logic code to
reference views from the layout using their view ids.

Layout file (`main.layout`):
```text
LinearLayout {
    // and edittext with id `myEditText`
    EditText (hint: "Type a text here!"): myEditText,
    
    // button with id `myButton`
    Button (text: "Hello world"): myButton,
}
```

Logic file (`main.logic`):
```text
string text

// bind an onClick event to our button
myButton.onClick {
    // do UI operations with them
    text = myEditText.getText()
    myButton.setText(text)
}
```

Project configuration (`swproj.toml`):
```toml
# ...

[activity.main]
logic = "main.logic"
layout = "main.layout"
```

### Views and attributes

View-independent attributes are:
 - `height`: value can either be `match_parent`, `wrap_content` or an arbitrary integer that denotes a fixed height in dp
 - `width`: same as height
 - `weight`: an integer that sets the weight of the view
 - `weight_sum`: an integer that sets the weight sum of the view
 - `background_color`: a hex value with any of these formats: `ffffff`, `#ffffff`, `ffffffff`, `#ffffffff` that sets the background color of the view
 - `layout_gravity`: can be any of:
   - `center_horizontal`
   - `center_vertical`
   - `center`
   - `left`
   - `right`
   - `top`
   - `bottom`
     or them combined separated with `|`. e.g. `left|bottom` (spaces are ignored).<br/>
     Specifying opposite gravity values together (e.g. `left|right`) will result in an error.
 - `padding`: an integer in dp that sets the padding in all directions
 - `padding_top`: an integer in dp that sets the padding on the top of the view
 - `padding_bottom`: an integer in dp that sets the padding on the bottom of the view
 - `padding_left`: an integer in dp that sets the padding on the left of the view
 - `padding_right`: an integer in dp that sets the padding on the right of the view
 - `padding_horizontal`: an integer in dp that sets the padding on the left and right of the view
 - `padding_vertical`: an integer in dp that sets the padding on the top and bottom of the view
 - `margin`: an integer in dp that sets the margin in all directions
 - `margin_top`: an integer in dp that sets the margin on the top of the view
 - `margin_bottom`: an integer in dp that sets the margin on the bottom of the view
 - `margin_left`: an integer in dp that sets the margin on the left of the view
 - `margin_right`: an integer in dp that sets the margin on the right of the view
 - `margin_horizontal`: an integer in dp that sets the margin on the left and right of the view
 - `margin_vertical`: an integer in dp that sets the margin on the top and bottom of the view

View-specific attributes in sketchware and supported by rafflesia are:
 - <details><summary><code>LinearLayout</code></summary>
   Attributes:
   <ul>
     <li><code>orientation</code>: <code>vertical</code> / <code>horizontal</code></li>
     <li><code>gravity</code>: can be any of <ul>
       <li><code>center_horizontal</code></li>
       <li><code>center_vertical</code></li>
       <li><code>center</code> (mix of both <code>center_horizontal</code> and <code>center_vertical</code></li>
       <li><code>left</code></li>
       <li><code>right</code></li>
       <li><code>top</code></li>
       <li><code>bottom</code></li>
     </ul>
     or them combined separated with <code>|</code>. e.g: <code>left|bottom</code> (spaces are ignored)<br/>
     Specifying opposite gravity values together (e.g. <code>left|right</code>) will result in an error.
     </li>
   </ul>
   </details>
 - <details><summary><code>Button</code></summary>
   Attributes:
   <ul>
     <li><code>text</code>: a text for the button. default is <code>Button</code></li>
     <li><code>text_color</code>: a hex value with any of these formats: <code>ffffff</code>, <code>#ffffff</code>, <code>ffffffff</code>, <code>#ffffffff</code> that sets the color of the text of the button</li>
     <li><code>text_size</code>: an integer that sets the size of the text of the button in dp. default is 12</li>
     <li><code>text_style</code>: can be any of <code>bold</code>, or <code>italic</code> or them combined separated with <code>|</code>. e.g. <code>bold|italic</code></li>
   </ul>
   </details>
 - <details><summary><code>TextView</code></summary>
   Attributes:
   <ul>
     <li><code>text</code>: a text for the button. default is <code>Button</code></li>
     <li><code>text_color</code>: a hex value with any of these formats: <code>ffffff</code>, <code>#ffffff</code>, <code>ffffffff</code>, <code>#ffffffff</code> that sets the color of the text of the textview</li>
     <li><code>text_size</code>: an integer that sets the size of the text of the button in dp. default is 12</li>
     <li><code>single_line</code>: a boolean that restricts the textview to be able to only have a single line if true. default is false</li>
     <li><code>text_font</code>: a font reference that sets the font of this textview. <b>please do note that resource management is unimplemented, this attribute won't work</b>. default is sketchware's <code>default_font</code></li>
     <li><code>text_style</code>: can be any of <code>bold</code>, or <code>italic</code> or them combined separated with <code>|</code>. e.g. <code>bold|italic</code></li>
     <li><code>lines</code>: an integer that restricts the amount of lines that can be displayed in the textview.</li>
   </ul>
   </details>
 - <details><summary><code>EditText</code></summary>
   Attributes:
   <ul>
     <li><code>text</code>: a text for the button. default is <code>Button</code></li>
     <li><code>text_color</code>: a hex value with any of these formats: <code>ffffff</code>, <code>#ffffff</code>, <code>ffffffff</code>, <code>#ffffffff</code> that sets the color of the text of the edittext</li>
     <li><code>text_size</code>: an integer that sets the size of the text of the button in dp. default is 12</li>
     <li><code>single_line</code>: a boolean that restricts the edittext to be able to only have a single line if true. default is false</li>
     <li><code>text_font</code>: a font reference that sets the font of this edittext. <b>please do note that resource management is unimplemented, this attribute won't work</b>. default is sketchware's <code>default_font</code></li>
     <li><code>text_style</code>: can be any of <code>bold</code>, or <code>italic</code> or them combined separated with <code>|</code>. e.g. <code>bold|italic</code></li>
     <li><code>lines</code>: an integer that restricts the amount of lines that can be displayed in the edittext.</li>
     <li><code>hint</code>: a text that sets the hint of this edittext. default is an empty string</li>
     <li><code>hint_color</code>: a hex value with any of these formats: <code>ffffff</code>, <code>#ffffff</code>, <code>ffffffff</code>, <code>#ffffffff</code> that sets the color of the hint of the edittext</li>
     <li><code>ime_option</code>: an ime option value that changes the "enter" button in soft keyboards.
       can be any of these values:
       <ul>
         <li><code>normal</code></li>
         <li><code>none</code></li>
         <li><code>go</code></li>
         <li><code>search</code></li>
         <li><code>send</code></li>
         <li><code>next</code></li>
         <li><code>done</code></li>
       </ul>
     </li>
     <li><code>input_type</code>: an input type that restricts the user from entering different types of values.
       can be any of these values:
       <ul>
         <li><code>decimal</code></li>
         <li><code>signed</code></li>
         <li><code>decimal_signed</code></li>
         <li><code>text</code></li>
         <li><code>password</code></li>
         <li><code>phone</code></li>
       </ul>
   </li>
   </ul>
   </details>
 - <details><summary><code>ImageView</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>
 - <details><summary><code>WebView</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>
 - <details><summary><code>ProgressBar</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>
 - <details><summary><code>ListView</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>
 - <details><summary><code>Spinner</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>
 - <details><summary><code>CheckBox</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>
 - <details><summary><code>ScrollView</code></summary>
   Attributes:
   <ul>
     <li><code>orientation</code>: <code>vertical</code> / <code>horizontal</code></li>
     <li><code>gravity</code>: can be any of <ul>
       <li><code>center_horizontal</code></li>
       <li><code>center_vertical</code></li>
       <li><code>center</code> (mix of both <code>center_horizontal</code> and <code>center_vertical</code></li>
       <li><code>left</code></li>
       <li><code>right</code></li>
       <li><code>top</code></li>
       <li><code>bottom</code></li>
     </ul>
     or them combined separated with <code>|</code>. e.g: <code>left|bottom</code> (spaces are ignored)<br/>
     Specifying opposite gravity values together (e.g. <code>left|right</code>) will result in an error.
     </li>
   </ul>
   </details>
 - <details><summary><code>Switch</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>
 - <details><summary><code>SeekBar</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>
 - <details><summary><code>CalendarView</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>
 - <details><summary><code>Fab</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>
 - <details><summary><code>AdView</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>
 - <details><summary><code>MapView</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>