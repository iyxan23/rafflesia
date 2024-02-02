# Padma Reform

Padma is modeled after the model of which most programming languages are derived from.
It is definitely a cool idea but it may not suit padma's original intention, which
is basically a compile-time blocks generator.

There's this one big problem where the user may not know that doing these function
calls doesn't actually call a function:

```
doSomething(): s {
  < do(something(), idk());
}
```

Let's say do has the signature of `do(s, d): s`, it takes a `string` as its first
parameter and a `number` as its second parameter.

The user here uses these definitions as if they are regular functions, but they're
actually not. Padma would resolve doSomething to be along the lines of:

```
#smt1
#smt2
 | generated from from something()

#idk1
#idk2
#idk3
 | generated from idk()

#do (#smt3, #idk4)
 | calling the #do with the return blocks of `something` and `idk`
```

The thing is that, we will never know that something that `idk()` do might
change the behavior of `something()`. Which the user will never expect to happen!
If they're just casually coding with their natural style.

This isn't a problem with how padma handles function calls, but rather it's
a fundamental design issue in the defs language itself. Because it appeared
as if its a regular programming language the user are familiar with. It should
not be because it is not a language that is interpreted, but a language to
describe what blocks to be placed.

## What I think looks good

Here's a brief idea, I'm going to write more later:

```
doSomething(s): s {
  hmm(@1, @1);

  $out1 = functions("mmm");
  $out2 = mmh(@1, "what", 10);

  otherFunc($out1, $out2);

  // $out1 and $out2 can't be used anymore
} : #do(@1);
```

Here we could:

- Know the orders of functions arguments to be executed
- Be certain about the output of the functions ($ variables can only be used once)
- Make sure that the return block always gets placed last, it's not a statement.
