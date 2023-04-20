The `padma` function notation to represent a function signature.

> Block types (s, d, b) can be seen on [Iyxan23/sketchware-data:data/block-opcodes.md](https://github.com/Iyxan23/sketchware-data/blob/main/data/block-opcodes.md)

## Defining functions

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

## Writing code

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

More complex expressions on bindings

Specifying a return type to a binding implicitly tells the resolver that
the statement after that is treated as a return statement.

```
// name = "hello world"; // error: literal is not a statement
name: s = "hello world";

//    -- might need a way to ignore arguments in bindings?
// name(s): s = "hello world"; // error in resolver: literal can't take an argument

// calling another function
process(d): s = function;
process(d): s = function(@0);
process(d): s = #lorem(function(), @0); // complex expressions
d.doSomething(b): s = @@.lorem(@0);
d.doSomething(b): s = @@.lorem; // smart argument choosing, lorem will be invoked with (@0) and not (@@, @0).
d.doSomething(b): s = doOtherThing(@@).lorem; // lorem will be invoked with (@@, @0)
d.doSomething: s = #lorem; // < #lorem(@@, @0);

// converts into

name(): s { < "hello world"; }
process(d): s { < function(@0); }
process(d): s { < function(@0); }
d.doSomething(b): s { < @@.lorem(@0); }
d.doSomething(b): s { < @@.lorem(@0); }
d.doSomething(b): s { < doOtherThing(@@).lorem(@@, @0); }
d.doSomething(b): s { < #lorem(@@, @0); }
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