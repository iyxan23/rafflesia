The `padma` function notation to represent a function signature.

> Block types (s, d, b) can be seen on [Iyxan23/sketchware-data:data/block-opcodes.md](https://github.com/Iyxan23/sketchware-data/blob/main/data/block-opcodes.md)

### Defining functions

```text
// regular functions
name() { ..blocks.. }
name(s) { ..blocks.. }

// returning functions
name(): d { ..blocks.. }
name(s): d { ..blocks.. }
name(s, s): d { ..blocks.. }

// binding
name = block;
name: d = block;
name(s): d = block;
name(s, s): d = block;
name(s, s): d = block(@1, @0); // <- flipping arguments

// methods
d.name() { ..blocks.. }
d.name(s) { ..blocks.. }
d.name(s): d { ..blocks.. }
d.name(s, s): d { ..blocks.. }

// binding methods
d.name = block;
//   and so on..
```

### Writing code

Raw block opcodes has a `#` prefix on them.

```text
name() {
    #opcode();
}
```

Pass arguments to them as literals, or from the function argument value.

```text
name(s) {
    #opcode("hello world", @0); // <- the `@0` takes the 0th argument of `name()` (which is an `s` [string])
}
```

Create methods on types.

```text
d.name() {
    #opcode(@@); // <- the `@@` takes `this` value (`d` type)
}
```

Returning values on functions.

```text
getString(): s {
    < "hello world";
}

d.toString(): s {
    < #toString(@@);
}
```

Bindings to automatically "bind" arguments with the opcode specified.

```
name = #block;
name(s) = #block;
name(s): s = #block;
name(s, s) = #block;
name(s, s) = #block(@1, @0); // <- manually specifying arguments
d.name(s) = #block;
d.name(s) = #block(@0, @@);

// the code above is equivalent to

name() { #block(); }
name(s) { #block(@0); }
name(s): s { < #block(@0); }
name(s, s) { #block(@0, @1); }
name(s, s) { #block(@1, @0); }
d.name(s) { #block(@@, @0); }
d.name(s) { #block(@0, @@); }

// IMPORTANT NOTE
// ===
// When using bindings, you really need to make sure you know that
// the types of the function that you want to bind to an opcode has
// the same types.

// for example, the opcode `toString` has one parameter of number
// and returns a string.
// ---
// then the bind definition must be `toString(n): s = #toString`.
// or creating a method, `n.toString(): s = #toString`.
```

Calling other padma functions inside functions. Recursive calls aren't allowed.

```
doSomething(d, d): s {
    doOtherThing(@0.toString());
    doOtherThing(@1.toString());
    #block(@0, @1);

    // v !disallowed! v
    // doSometihng(..);

    < returning();
}

doOtherThing(s) { ... }
returning(): s { ... }
d.toString(): s = toString;
```

#### Nested?

Currently there is no way to use nested functions, because it would be hard to return a value inside
these nested if statements. It is currently not possible because these blocks translate into raw blocks
and we can't have an "inline if statement", for example.

One seemingly possible workaround is to implicitly transfer where the caller used the function inside the nest.

So, for example (imaginary syntax):

```
// takes a boolean and return a human string of it
getString(b): s {
    #ifElse(@0) {
        // first nest
        #myBlock();
        < "It's true!";
    } {
        // second nest
        #myOtherBlock();
        < "It's false!";
    };
}
```

If we use this in rafflesia as:

```text
my_str = getString(true);
toast(getString(true));
```

This code would be generated as (pseudo-code block representation syntax):

```
// the variable assignment
0: ifElse "true" substack1=2 substack2=4
1: myBlock
2: setVarStr "my_str" "It's true!"
3: myOtherBlock
4: setVarStr "my_str" "It's false!"

// the toast function call
5: ifElse "true" substack1=7 substack2=9
6: myBlock
7: toast "It's true!"
8: myOtherBlock
9: toast "It's false!"
```

One problem that I found is what will happen if there are more code after the return statement? Like, returning a value early.

```
getString(b): s {
    #if(@0) {
        #myBlock();
        < "It's true!";
    };

    #myOtherBlock();
    < "It's false!";
}
```

```
my_str = getString(true);
```

Generated blocks (simplified syntax):
```
if "true" {
    myBlock
    setVarString "my_str" "It's true"
    // ??? 
}

myOtherBlock
setVarString "It's false!"
```

The problem is that padma defs files aren't real functions, they don't have a scope and can't return something.
They're more like macros to generate blocks. We shouldn't treat padma defs files as to defining functions,
but rather defining macros.

Perhaps we could disallow returns to happen inside nests?