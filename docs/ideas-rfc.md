# RFCs

Some things that I want to get implemented but also might need some thinking. I may or
may not just start implementing these lol.

## Rafflesia

Rafflesia RFCs

### Map type enforcement

Maps in sketchware aren't typed and they are just `HashMap<String, Object>`, this is a cool opportunity where
rafflesia could introduce its own restrictions where we will require to specify a map's type, but at the end, 
they compile to the same blocks as if they didn't have
types.

```
map<string> my_strings;

onCreate {
    my_strings.push("hello");
    my_strings.push(1); // error `map<string>.push(number)` is not implemented
}
```

### Moreblocks

I forgot to implement moreblocks, but they seem to be trivial to do so.

Here's a syntax I propose:

```
moreblock greet(name: string) {
    toast("Hello, " + name);
}

onCreate {
    greet("Iyxan23");
}
```

Moreblocks can't have a return type so it makes sense for us to not be able to specify a return type.

### Component types

Currently component types doesn't exist yet. But I mean why not, for fun? :>

```
firebasedb fdb;
gyroscope gs;

string name;

onCreate {
    gs.doSomething();
}

my_btn.onClick {
    name = fdb.getData("/somewhere/idk/");
}
```

### Flags

Having feature flags would be a cool idea, imagine having an a feature implemented,
but not yet stable, we could put it behind a feature flag, which people could use by enabling
the flag.

It'd be defined in the `swproj.toml` I suppose.

```toml
// ...

[project.compiler]
flags = [
    "padma-complex-types",
    "padma-nested",
    "rafflesia-complex-type-enforcement",
    // ...
]

// ...
```

## Padma

Padma RFCs

### Control types

It'd be cool to be able to loop or do some condition that runs at compile time to generate blocks dynamically.

```
doSomething() {
    !repeat(5) {
        #doSomething();
    }
}

// will output `#doSomething` calls five times.
```

We should be able to explicitly specify that a parameter only takes a literal, and it would be able to be calculated at compile-time.

```
runThisTimes(*d, *b) {
    !if(@1) {
        #makeThings(@0); // but should also be able to pass them as regular values
    }

    !repeat(@0) {
        #doSomething();
    }
}
```

### Ability to define expressions

Right now, padma serves as a way to define custom functions that takes custom blocks, but it is also
planned to be the de facto method of defining blocks in rafflesia; on the user side, or even in the compiler
side.

I have a plan to write a "standard library" where there are stadard padma definitions that gets loaded on every
projects (and maybe have an option to disable them). With this, I also would like to not make padma only
works for functions, but also for anything that relates to block code generation.

It'd be cool to be able to modify expressions like arithmetic expressions, or even variable assignment.

My proposed syntax:
```
// assignment
$b = b {
    // @0 acts as `$b`
    // @1 acts as the `b`
    #setVarBool(@0, @1);
}

// arithmetic
n + n: n {
    < #`+`(@0, @1);
}

// or better
n + n: n = #`+`;

// maybe indexing as well
l<s>[n]: s {
    // @@ acts as the `l<s>`
    // while @0 acts as the `n` indexer
    < #get(@@, @0);
}
```

### Complex Types

One important thing is to be able to use complex types (types that has a type parameter) things, they
are lists and maps.

Lists takes a type which they store, so we need a way to specify those types.

I'll take the good old approach of using chevrons:

```
l<s>.push(s) {
    // suppose we have an `addItem` block
    #addItem(@@, @0);
}
```

But wait what about maps? I guess that's blocked on "when-map-complex-type-enforcement-is-implemented"

But anyway the same should go for maps:

```
m<s>.insert(s, s) {
    // suppose we have an `insert` block
    #insert(@@, @0, @1);
}
```

and passing these complex types

```
doSomething(m<s>, s) {
    #foo(@0, @1);
    #bar(@0);
}
```

### View and component types

Since there are a lot of view and component types, we can't shorten them to one character. We might need
to specify the full name to get the type.

```
textview.setColor(d) {
    #actuallySetColor(@@, @d);
}

gyroscope.doSomething(d, d) {
    // ...
}

seeText(textview): s {
    < #something(@0);
}
```

### Passing variables

Since padma functions aren't "real functions", we should be able to pass something like a variable
directly, not by their value.

```
// something like this maybe?
setBoolVar($b) {
    #setVarBool(@0, true);
}
```

In rafflesia code:

```
boolean myBool

onCreate {
    // which one's better?
    setBoolVar(myBool);

    setBoolVar($myBool);
    setBoolVar(&myBool);
    // ^ these ones are clearer, we know that the variable is passed directly,
    //   and not as a value.

    setBoolVar(true); // <- error
}
```

### Nested?

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