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

### Views list

> todo

A list of views (and attributes) that exists in sketchware and are supported by rafflesia are:
 - <details><summary><code>LinearLayout</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>

 - <details><summary><code>Button</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>
 - <details><summary><code>TextView</code></summary>
   Attributes:
   <ul>
     <li></li>
   </ul>
   </details>
 - <details><summary><code>EditText</code></summary>
   Attributes:
   <ul>
     <li></li>
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
     <li></li>
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