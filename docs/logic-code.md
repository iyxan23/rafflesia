Each logic file represents a logic for an activity. The activity it bounded to it is defined in the `swproj.toml`
project configuration file. Its filename contributes nothing to the activity name.

```toml
[activity.main] # <- the name used for the activity
logic = "main.logic" # references to src/main.logic
layout = "main.layout"
```

## Structure of a logic file

Here's a simple example of a logic file

```text
 0 | number myNum
 1 | string myStr
 2 |
 3 | // hello rafflesia!
 4 |
 5 | onCreate {
 6 |     myNum = 1
 7 |     textview1.setText(toString(myNum))
 8 | }
 9 |
10 | map<string> strings
11 |
12 | button.onClick {
13 |     if myNum == 10 {
14 |         // show a warning!
15 |         toast("Limit exceeded!")
16 |     } else {
17 |         myNum = myNum + 1
18 |         myStr = edittext1.text
19 |
20 |         // save it to our map
21 |         strings[toString(myNum)] = myStr
22 |     }
23 | }
```

> Rafflesia uses `//` as its comment symbol, anything that precedes that symbol will be ignored by the compiler.

From the example above, a logic file has two scopes:
 - An outer scope
 - And an inner scope

### Outer scope

The outer scope is where you define variables and events

```text
 0 | number myNum
 1 | string myStr
   .
   .
10 | map<string> strings
   .
   .
```

#### Variables

Here you define variables by their type, and then their name.

```text
number myNum
string myStr
```

For complex types like maps and lists that can store multiple other types, you will need to specify another type.

The type that they store is written inside an angle bracket (`<...>`) after the type.

```text
list<number> nums    // a list of numbers
map<string> names    // a map of strings
```

The types we have on rafflesia are only 4:
 - `number`: A number
 - `string`: A text
 - `list<...>`: A list of something
 - `map<...>`: A map of something

> Components coming soon :l

#### Events

Events are blocks of code that gets executed at a certain point.

##### Activity events

Activity events are defined by their name and followed a curly brace block (`{ ... }`) where inside it is a block of
code called as the "Inner scope".

```text
onCreate {
    toast("Executed on create!")
}
```

There are a few activity events in sketchware that are called at different time points:
 - `onCreate`: runs at the creation of the activity
 - `onBackPressed`: when the user presses the back button
 - `onPostCreate`: when the activity has completed its startup
 - `onStart`: runs at the start of the activity (runs after the activity has created and has displayed)
 - `onResume`: when the activity got resumed (ex. after the activity over it has finished)
 - `onPause`: when the activity got paused (ex. when another new activity is presented to the user)

##### View events

There is another type of event other than activity events: View events.

View events are events that are bound to a view rather than an activity.

These events may include an `onClick` event for a `Button`, `onTextChanged` event for an `EditText` and so on.

It's currently unplanned on what view events are to be added in rafflesia's logic code as sketchware seems to have
dozens of them.

##### Component events

coming sOoOoON??

### Inner scope

The scope of the code inside events are called as the Inner scope. The inner scope is where everything moves.

```text
   .
   .
 5 | onCreate {
 6 |     myNum = 1
 7 |     textview1.setText(toString(myNum))
 8 | }
   .
   .
12 | button.onClick {
13 |     if myNum == 10 {
14 |         // show a warning!
15 |         toast("Limit exceeded!")
16 |     } else {
17 |         myNum = myNum + 1
18 |         myStr = edittext1.text
19 |
20 |         // save it to our map
21 |         strings[toString(myNum)] = myStr
22 |     }
23 | }
```

Each statement inside the inner scope must be preceded with a newline and cannot be one-lined.

Here, you can do logic operations:
 - Assigning and accessing a variable that is defined in the outer scope 
   ```text
   number myNum
   number myOtherNum
   
   onCreate {
       myNum = 5 * 20
       myOtherNum = myNum + 50
   }
   ```
 - Do operations with the UI using [global view access]()
   ```text
   number myNum
   
   onCreate {
       myNum = 50
       textview1.setText(toString(myNum * 10))
   }
   ```
 - Running a block of code if a condition is met or otherwise
   ```text
   number myNum
   
   onCreate {
       myNum = 5
   
       if myNum == 5 {
           toast("it works!")
       } else {
           toast("something is wrong")
       }
   }
   ```
 - Running a block of code a defined number of time
   ```text
   number myNum
   
   onCreate {
       myNum = 5
   
       // runs the block of code 10 times
       repeat myNum + 5 {
           toast("boom!")
       }
   }
   ```
 - Running a block of code indefinitely
   ```text
   onCreate {
       // runs this block of code over and over again
       // do not compile and run this
       forever {
           toast("dont")
       }
   }
   ```